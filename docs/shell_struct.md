# Shell Struct Architecture

The `Shell` struct is the core engine of seal. It owns all persistent shell state and exposes a single
`execute(&mut self, node: SyntaxNode) -> i32` entry point. The REPL, scripts, and Lua are all thin
frontends that call into `Shell`.

---

## Ownership Tree

```
Shell
 ├── env: Env
 │    ├── vars: HashMap<String, String>   ← exported environment variables
 │    └── cwd: PathBuf                    ← current working directory (tracked explicitly)
 │
 ├── jobs: JobTable
 │    ├── jobs: Vec<Job>                  ← all active/suspended jobs
 │    └── next_id: usize                  ← monotonic counter, never reuses IDs
 │
 ├── opts: ShellOpts
 │    ├── interactive: bool               ← REPL vs script mode
 │    ├── errexit: bool                   ← set -e: exit on non-zero
 │    ├── xtrace: bool                    ← set -x: print commands before running
 │    └── noglob: bool                    ← set -f: disable glob expansion
 │
 ├── last_exit: i32                       ← flat on Shell, accessed after every command
 ├── shell_pgid: Pid
 └── lua: Option<LuaEngine>              ← owns mlua::Lua, no back-reference to Shell
      └── lua: mlua::Lua
```

---

## Code Structures

### `Shell`

```rust
pub struct Shell {
    pub env:       Env,
    pub jobs:      JobTable,
    pub opts:      ShellOpts,
    pub last_exit: i32,
    pub(crate) lua: Option<LuaEngine>,
}

impl Shell {
    pub fn new() -> Self { ... }

    /// Single public entry point called by the REPL and scripts.
    pub fn execute(&mut self, node: SyntaxNode) -> i32 { ... }

    /// Convenience: lex + parse + execute a raw string.
    pub fn execute_str(&mut self, input: &str) -> i32 { ... }
}
```

---

### `Env` — Environment Subsystem

```rust
pub struct Env {
    vars: HashMap<String, String>,
    cwd:  PathBuf,
}

impl Env {
    pub fn get(&self, key: &str) -> Option<&str> { ... }
    pub fn set(&mut self, key: &str, val: &str) { ... }
    pub fn unset(&mut self, key: &str) { ... }

    /// Serialise vars into CString list suitable for execve(2).
    pub fn as_execve_list(&self) -> Vec<CString> { ... }

    pub fn cwd(&self) -> &Path { &self.cwd }
    pub fn set_cwd(&mut self, path: PathBuf) { self.cwd = path; }
}
```

> **Gotcha — cwd ownership:** Never call `std::env::set_current_dir` as the only record of the current
> directory. The shell process must track `cwd` explicitly in `Env` so that subshells, scripts, and Lua
> callbacks all see a consistent value without relying on kernel process state.

---

### `JobTable` — Job Control Subsystem

```rust
pub struct JobTable {
    jobs:    Vec<Job>,
    next_id: usize,   // monotonically increasing, never reused
}

impl JobTable {
    pub fn add(&mut self, job: Job) { ... }
    pub fn find_by_pgid(&self, pgid: Pid) -> Option<&Job> { ... }
    pub fn find_by_pgid_mut(&mut self, pgid: Pid) -> Option<&mut Job> { ... }
    pub fn remove_done(&mut self) { ... }
    pub fn next_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}
```

> **Gotcha — ID reuse:** The old code computed `job.id + 1` from the last job in the vec. When a job is
> removed, the next job gets a lower ID than expected, breaking `fg %N`. The `next_id: usize` counter on
> `JobTable` fixes this permanently.

---

### `ShellOpts` — Behavior Flags

```rust
pub struct ShellOpts {
    pub interactive: bool,  // true in REPL, false when running a script file
    pub errexit:     bool,  // set -e  — abort sequence on first non-zero exit
    pub xtrace:      bool,  // set -x  — print "+ cmd args" before each command
    pub noglob:      bool,  // set -f  — disable pathname expansion (globbing)
}

impl Default for ShellOpts {
    fn default() -> Self {
        Self {
            interactive: true,
            errexit:     false,
            xtrace:      false,
            noglob:      false,
        }
    }
}
```

> **Gotcha — subshell option inheritance:** When a subshell `(...)` is created, it should receive a
> **clone** of `ShellOpts` from the parent. Because it is a plain struct with no heap allocation, this is
> free. Any `set -e` or `set -x` change inside the subshell must NOT propagate back to the parent.

---

### `last_exit: i32` — Why It Is Flat

`last_exit` is read after **every single command** to evaluate `&&`, `||`, `set -e`, and `$?`.
Nesting it inside another struct adds indirection for zero benefit.

```rust
// In the executor, after every node:
self.last_exit = self.execute_pipeline(cmds);

// In parse_and_or execution:
SyntaxNode::And { left, right } => {
    let code = self.execute(left);
    if code == 0 { self.execute(right) } else { code }
}
```

---

### `LuaEngine` — Why It Is Safe to Own Inside Shell

The apparent problem: `Shell` owns `LuaEngine`, but `LuaEngine` needs to call back into `Shell`.
This looks circular but is not, because `LuaEngine` stores **zero references** to `Shell`.
The borrow only exists inside a `mlua::Lua::scope` call at the moment a Lua script runs:

```rust
pub struct LuaEngine {
    lua: mlua::Lua,  // just the VM — no reference to Shell anywhere
}

impl LuaEngine {
    /// Shell is passed IN here, not stored inside LuaEngine.
    pub fn run_script(&self, code: &str, shell: &mut Shell) -> mlua::Result<()> {
        self.lua.scope(|scope| {
            // The borrow of `shell` lives only for this closure's duration.
            let seal_run = scope.create_function_mut(|_, cmd: String| {
                Ok(shell.execute_str(&cmd))
            })?;

            let seal_table = self.lua.create_table()?;
            seal_table.set("run", seal_run)?;
            self.lua.globals().set("seal", seal_table)?;

            self.lua.load(code).exec()
        })
    }
}
```

> **Gotcha — Lua engine initialisation timing:** `LuaEngine` must be constructed **after** `Shell::new()`
> returns. It cannot be created inside `Shell::new()` while the shell is being built, because `scope`
> requires a live `&mut Shell` reference. Store it as `Option<LuaEngine>`, initialise it after
> construction, and check `shell.lua.is_some()` before running Lua scripts.

---

## What Does NOT Belong in Shell

| Thing | Where It Belongs | Reason |
|---|---|---|
| Pipe file descriptors | Local variables in `execute_pipeline` | Temporary, alive only during fork |
| Current `pgid` during fork | Local to executor | Temporary, not persistent state |
| Rustyline / readline state | `Repl` struct | Frontend concern, not shell logic |
| Lexer / parser state | Stateless free functions | No state needed between calls |
| `SyntaxNode` being executed | Stack frame of `execute()` | Consumed, not stored |
| REPL prompt string | `Repl` or prompt module | Presentational, not execution state |

---

## The Migration Path from `Rc<RefCell<ShellState>>`

The old `ShellStateType = Rc<RefCell<ShellState>>` is eliminated by:

1. **Creating `Shell`** with the sub-structs above
2. **Passing `&mut Shell`** into every executor function instead of cloning an `Rc`
3. **Deleting `types.rs`** — it only contains the `Rc<RefCell<...>>` type alias
4. **Deleting `models.rs`** old types (`Token`, `Pipeline`, `Command`) after the executor is rewritten
   to use `SyntaxNode` from the parser

The constraint that makes this work: the executor must be called **synchronously** from `Shell::execute`.
No executor function should store `&mut Shell` across a `fork()` boundary — the child process gets its
own copy of the shell's memory and the parent continues with its own `&mut Shell`.

---

## Terminal Ownership

Terminal ownership is **NOT** a REPL concern. The REPL has one assumption:

> *"By the time I call `readline()`, the shell already has the terminal."*

It never calls `tcsetpgrp`. It just reads.

### Who Owns What

```
Shell::new() / main
  ├── setpgid(getpid(), getpid())      ← shell becomes its own process group
  └── tcsetpgrp(STDIN, shell_pgid)    ← shell claims the terminal at startup

execute_pipeline (foreground job)
  ├── [parent] setpgid(child, child)  ← BEFORE tcsetpgrp — fixes the race condition
  ├── [child]  setpgid(0, 0)          ← child also sets this (both sides of fork)
  ├── [parent] tcsetpgrp(STDIN, job_pgid)   ← give terminal to job
  ├── waitpid(...)
  └── [parent] tcsetpgrp(STDIN, shell_pgid) ← take terminal BACK

JobTable::fg (fg builtin)
  ├── tcsetpgrp(STDIN, job_pgid)      ← give terminal to resumed job
  ├── kill(-pgid, SIGCONT)            ← wake the stopped job
  ├── waitpid(...)
  └── tcsetpgrp(STDIN, shell_pgid)   ← take terminal back

Repl::readline()
  └── (just reads — assumes terminal is already owned by shell)
```

### `shell_pgid` Field

The executor needs to know the shell's own pgid to call `tcsetpgrp` after a job
finishes. This is shell state, not executor-local state. Add it to `Shell`:

```rust
pub struct Shell {
    pub env:       Env,
    pub jobs:      JobTable,
    pub opts:      ShellOpts,
    pub last_exit: i32,
    pub shell_pgid: Pid,             // ← needed for tcsetpgrp after every foreground job
    pub(crate) lua: Option<LuaEngine>,
}
```

Set it once in `Shell::new()` and never change it:

```rust
impl Shell {
    pub fn new() -> Self {
        let pid = Pid::from_raw(unsafe { nix::libc::getpid() });
        unsafe {
            nix::libc::setpgid(pid.as_raw(), pid.as_raw());
            nix::libc::tcsetpgrp(nix::libc::STDIN_FILENO, pid.as_raw());
        }
        Self {
            shell_pgid: pid,
            // ...
        }
    }
}
```

> **Gotcha — the setpgid/tcsetpgrp race:** Both the parent AND the child must call `setpgid` after
> `fork()`. The parent calls it to guarantee the group exists before `tcsetpgrp`. The child calls it
> in case it runs first. Without both calls, `tcsetpgrp` in the parent can fail silently with EPERM
> because the group doesn't exist yet, leaving the terminal attached to the shell — so Ctrl+C hits
> the shell instead of the child. This is the root cause of the original terminal ownership bug.
