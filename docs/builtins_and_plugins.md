# Builtins & Plugin Architecture Guide

This document defines the architectural specification of Seal's Builtin and Plugin System, detailing how native Rust commands and Lua scripts register as first-class citizens in the shell.

---

## 1. Architectural Role & Responsibilities

In traditional shells, builtins are hardcoded functions baked directly into the interpreter loop. In Seal, builtins follow a **Dynamic Plugin Registry Pattern**:

```text
BuiltinRegistry (HashMap<String, Box<dyn Builtin>>)
 ├── Native Rust Builtins    ("cd", "echo", "export", "exit", "pwd", "jobs")
 └── Dynamic Lua Plugins     ("greet", "git-helper", user-registered scripts)
```

### Key Architectural Boundaries:
- **Identical Interface:** The execution engine does not know or care whether a command was written in Rust or Lua. Both implement the exact same `Builtin` trait.
- **In-Process Execution:** Builtins run inside the main shell process. This allows commands like `cd`, `export`, and `exit` to directly mutate shell state without being lost in a subshell fork.
- **Pipeline Compatibility:** Builtins do not write directly to OS `std::io::stdout`. They write to a redirected stream passed through `ShellContext`, allowing them to participate in arbitrary pipelines (e.g. `echo hi | grep h` or `cat file | my_plugin`).

---

## 2. The Core Plugin Architecture

### The `Builtin` Trait Contract
Every builtin registers:
1. **Name:** The string command used to invoke it at the prompt.
2. **Help Documentation:** Short description string.
3. **Execution Logic:** Receives arguments and `ShellContext`, returns an integer exit code (`0` for success).

### The Registry
The `BuiltinRegistry` maintains a map of command names to boxed trait objects (`HashMap<String, Box<dyn Builtin>>`). When a command is dispatched:
1. The shell checks `registry.get(program_name)`.
2. **If found:** Executes the builtin in-process.
3. **If not found:** Searches `$PATH` for an external binary to `fork`/`exec`.

---

## 3. Lua Plugins as Builtins

A major strength of this architecture is that user Lua scripts can register custom commands into the shell at startup (e.g. in `~/.config/seal/init.lua`):

```lua
seal.register_builtin("greet", function(args)
    local name = args[1] or "world"
    print("Hello, " .. name .. "!")
    return 0
end)
```

Once registered, `greet` is treated identically to `cd` or `echo`:
* It is recognized by tab-completion.
* It can be run at the prompt: `greet Alice`.
* It can be piped: `greet Alice | tr 'a-z' 'A-Z'`.

---

## 4. Specific Things to Keep an Eye On (Gotchas & Pitfalls)

When implementing or modifying builtins and plugins, watch out for these subtle edge cases:

### 1. The I/O Redirection Hazard (Direct `println!` vs `ctx.stdout`)
* **The Pitfall:** Writing directly to Rust's `println!()` or `std::io::stdout()` inside a builtin.
* **Why it breaks:** In a pipeline like `echo "hello" | tr 'a-z' 'A-Z'`, stdout must be redirected to the write end of a pipe. If a builtin writes directly to standard output, its output bypasses the pipeline entirely and splashes directly onto the terminal screen!
* **Watch Out:** Builtins must **always write to `ctx.stdout`** (or use `dup2` before calling the handler).

### 2. State Mutation Requires Parent Process Execution
* **The Pitfall:** Spawning builtins like `cd`, `export`, or `exit` inside a forked child process.
* **Impact:** In UNIX, child processes inherit a copy of the parent's environment and working directory, but **cannot modify the parent's state**. If `cd` runs inside a forked child, the child changes directory and immediately exits—leaving the shell still in the old directory!
* **Watch Out:** 
  - Commands that alter shell state (`cd`, `export`, `exit`) must run **directly in the parent shell process**.
  - When part of a multi-command pipeline (`cat file | cd foo`), standard shells run them in a subshell where directory changes are intentionally isolated.

### 3. Lua Function Garbage Collection (`RegistryKey`)
* **The Pitfall:** Storing a Lua function pointer in Rust as a temporary value.
* **Impact:** Lua features an automatic garbage collector. If a function passed to `seal.register_builtin` is not anchored in Lua's internal registry, Lua's GC will sweep it away, causing subsequent command calls to crash or segfault!
* **Watch Out:** When Lua passes a callback to Rust, Rust must anchor it using `mlua::Lua::create_registry_value()`. This produces an integer `RegistryKey` stored in `LUA_REGISTRYINDEX`, guaranteeing the callback remains valid for the lifetime of the shell.

### 4. Argument Index Offsets (`args[0]` vs `args[1]`)
* **The Pitfall:** Inconsistency between how Rust and Lua index command line arguments.
* **Rules to maintain:**
  - In Rust: `args[0]` is the command name itself (e.g. `"cd"`), and `args[1]` is the first parameter.
  - In Lua: Arrays are 1-indexed. If Rust passes `args[1..]` to the Lua function, `args[1]` in Lua is the user's first argument.
* **Watch Out:** Always document whether an argument slice passed to a plugin includes the command name (`args[0]`) or only the parameters.

### 5. Precedence: Builtins Must Shadow `$PATH`
* **The Pitfall:** Searching `$PATH` before checking the builtin registry.
* **Impact:** If the user has a binary `/bin/echo` on disk, searching disk first would run the external `/bin/echo` instead of the fast in-process `echo` builtin, wasting execution time on `fork` and `exec`.
* **Watch Out:** Builtins must always take precedence over external binaries with the same name.
