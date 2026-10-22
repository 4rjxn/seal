# Lua API & Scripting Architecture Guide

This document defines the architectural specification of Seal's Lua Scripting Engine (`lua_engine.rs`), detailing the `seal.*` API contract, safe execution patterns, and critical integration gotchas.

---

## 1. Architectural Role & Responsibilities

The Lua Engine serves as a programmatic **Frontend** to the Seal Core Engine. It allows users to write shell startup configurations (`~/.config/seal/init.lua`), automate complex tasks, and register custom shell plugins.

```text
┌────────────────────────────────────────────────────────┐
│                   LUA SCRIPTING FRONTEND               │
│                                                        │
│   • init.lua (Config, Custom Keybinds, Colors)         │
│   • Standalone Scripts (Automations, Build pipelines)  │
│   • Custom Builtin Plugins                             │
└───────────────────────────┬────────────────────────────┘
                            │
              Calls through namespaced API
                            ▼
┌────────────────────────────────────────────────────────┐
│               SEAL LUA BRIDGE (`seal.*`)               │
│                                                        │
│   seal.run()      seal.capture()   seal.register_builtin()│
│   seal.env        seal.cd()        seal.cwd()          │
└───────────────────────────┬────────────────────────────┘
                            │
               Borrows &mut ShellState via scope
                            ▼
┌────────────────────────────────────────────────────────┐
│                  SEAL CORE ENGINE (lib.rs)             │
└────────────────────────────────────────────────────────┘
```

### Key Architectural Boundaries:
- **No Direct ShellState Ownership:** The Lua engine does not own `ShellState`. When executing a Lua script, it borrows `&mut ShellState` using `mlua::Lua::scope()`.
- **Namespaced API (`seal.*`):** All shell capabilities reside inside a dedicated `seal` table. Lua's global namespace (`_G`) is never polluted.
- **Structured Data Flow:** Command execution returns rich objects with exit codes, stdout, and stderr rather than discarding information.

---

## 2. The `seal.*` API Contract

The Lua bridge exposes five primary capabilities:

| API | Signature | Description |
|---|---|---|
| `seal.run` | `(string \| table) -> integer` | Executes command with terminal I/O. Returns process exit code (`0` = success). |
| `seal.capture` | `(string \| table) -> Result` | Executes command and captures I/O. Returns `{ status, stdout, stderr }`. |
| `seal.cd` | `(string) -> nil` | Updates the shell's active working directory. |
| `seal.cwd` | `() -> string` | Returns the current working directory path. |
| `seal.env` | `table` | Metatable allowing reading and setting environment variables (`seal.env.KEY = val`). |
| `seal.register_builtin` | `(string, function) -> nil` | Registers a Lua function as a new shell command. |

---

## 3. Specific Things to Keep an Eye On (Gotchas & Pitfalls)

When implementing or extending the Lua bridge, watch out for these subtle edge cases:

### 1. The Shell Injection Trap (String Concatenation vs. Array Syntax)
* **The Pitfall:** Only allowing Lua to pass raw command strings (`seal.run("rm " .. filename)`).
* **Why it breaks:** If `filename` contains spaces (e.g. `My Photos.png`), the shell interprets it as two arguments (`My` and `Photos.png`). Worse, if `filename` contains semicolons or pipes (e.g. `file.txt; rm -rf /`), it triggers accidental shell injection!
* **Watch Out:** The API must accept **both** strings and tables of arguments:
  ```lua
  -- Unsafe if filename has special chars:
  seal.run("rm " .. filename)

  -- Safe! Arguments are passed directly without going through string parsing:
  seal.run({"rm", "-f", filename})
  ```

### 2. The Silent Failure Trap (Discarding Exit Codes & Stderr)
* **The Pitfall:** Having `run()` return nothing, or having `capture()` return only a single stdout string.
* **Why it breaks:** Scripting requires error checking. If `cargo test` fails, a script must be able to branch on `status ~= 0`. If `git status` fails, the error message printed to `stderr` must not be thrown away.
* **Watch Out:**
  - `seal.run()` must **always return the integer exit code**.
  - `seal.capture()` must **always return all three components**: `{ status = code, stdout = "...", stderr = "..." }`.

### 3. State Desynchronization Between Lua and OS Environment
* **The Pitfall:** Accessing Lua's built-in `os.getenv` or `os.execute("cd ...")`.
* **Impact:** 
  - `os.execute("cd /tmp")` executes in an isolated subshell. The parent shell's working directory remains unchanged!
  - `os.getenv` queries the OS process environment table, which does not reflect environment variables added dynamically inside Seal's `state.env_vars`.
* **Watch Out:** Lua scripts must use `seal.cd()` and `seal.env.*` so all mutations synchronize directly with Seal's `ShellState`.

### 4. Zero-Cost Lifetime Scoping with `mlua::Lua::scope`
* **The Pitfall:** Storing `&mut ShellState` inside a long-lived struct or resorting to `Rc<RefCell<ShellState>>`.
* **How `scope` solves it:** `mlua`'s `scope` construct allows registering callbacks that borrow local stack variables for the duration of a script execution block.
* **Watch Out:** Any closures registered inside `lua.scope(|scope| { ... })` cannot be called after the `scope` block exits. For callbacks that *must* live forever (like `seal.register_builtin`), use `mlua::Lua::create_registry_value()` to store them in the Lua registry.

### 5. String Trimming on Captures
* **The Pitfall:** Leaving trailing newlines on captured command output.
* **Impact:** Commands like `git rev-parse --abbrev-ref HEAD` or `whoami` output their text followed by a newline `\n`. If a user does `local branch = seal.capture("git branch ...").stdout` and concatenates it (`"origin/" .. branch`), the newline breaks the string!
* **Watch Out:** Provide a convenient helper or ensure `capture` documentation clearly notes whether output contains raw newlines so users know to use `:gsub("%s+$", "")` or provide a trimmed accessor.
