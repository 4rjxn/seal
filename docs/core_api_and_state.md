# Core Engine & State Management Architecture Guide

This document defines the architectural specification of the Seal Core Engine (`lib.rs`), its state management model, and the critical edge cases to watch out for.

---

## 1. Architectural Role & Responsibilities

The Core Engine represents the central nervous system of Seal. It encapsulates all state, builtins, process execution, and configuration behind a unified, embeddable `Shell` interface in `lib.rs`.

```text
┌────────────────────────────────────────────────────────┐
│                      FRONTENDS                         │
│       Interactive REPL         Lua Scripting Engine    │
│      (stdin/stdout/tty)       (embedded scripts, init) │
└───────────────────────┬────────────────────────┬───────┘
                        │                        │
                        ▼                        ▼
┌────────────────────────────────────────────────────────┐
│                   SEAL ENGINE (lib.rs)                 │
│                                                        │
│   • ShellState: env vars, cwd, jobs, last_status       │
│   • BuiltinRegistry: cd, echo, export, plugins...      │
│   • AST Execution Engine & Pipeline Runner             │
└────────────────────────────────────────────────────────┘
```

### Key Architectural Boundaries:
- **Shell as a Library:** The entire engine lives in `lib.rs`. `main.rs` is merely a thin binary wrapper (~30 lines) that drives the interactive terminal REPL.
- **Single Source of Truth:** Both the REPL and Lua scripts execute through the exact same engine methods. Builtins modify the same environment and working directory regardless of where they were triggered.
- **Single Ownership Model:** The `Shell` instance is the sole owner of all mutable state. No subsystems hold circular references.

---

## 2. State Architecture: Eliminating `Rc<RefCell>`

### The Reference Cycle Hazard
In the original codebase, `ShellState` held `Option<Rc<LuaEngine>>`, while `LuaEngine` held `Rc<RefCell<ShellState>>`. This created a memory cycle that prevented clean destruction and forced runtime borrow-checking (`.borrow_mut()`) across the entire repository.

### The New Ownership Hierarchy
```text
Shell (Root Owner)
 ├── ShellState          (Pure data: env, cwd, jobs, exit status)
 ├── BuiltinRegistry     (Map of builtin handlers)
 └── LuaEngine           (Lua state, borrows &mut ShellState on demand)
```

1. **No `Rc` or `RefCell` in Core:** `ShellState` is a standard struct owned by value.
2. **Deterministic Borrowing:** Execution functions borrow `&mut ShellState` down the call stack. Rust's compiler guarantees at compile time that no aliasing or race conditions can occur.
3. **Lua Interfacing via `scope`:** When executing a Lua script, the engine uses `mlua::Lua::scope()` to temporarily borrow `&mut ShellState` exclusively for the duration of the script call.

---

## 3. The Dedicated Expansion Stage

Syntax parsing produces a raw, literal AST. Between parsing and execution lies the **Expansion Stage**:

```text
AST Node ──▶ [Expansion Stage] ──▶ Executable Command
                     │
              Uses &ShellState
```

### Expansion Order (POSIX Contract):
1. **Tilde Expansion (`~`):** Replaces a leading `~` or `~/` with `$HOME`.
2. **Parameter Expansion (`$VAR`, `$?`):** Replaces `$NAME` with the corresponding variable from `state.env_vars`, and `$?` with `state.last_status`.
3. **Globbing (`*.rs`, `src/**/*.rs`):** Matches unquoted wildcards against the actual filesystem relative to `state.current_dir`.

---

## 4. Specific Things to Keep an Eye On (Gotchas & Pitfalls)

When implementing or modifying the core engine, watch out for these subtle edge cases:

### 1. The Runtime `BorrowMutError` Panic Hazard
* **The Pitfall:** Leaving `Rc<RefCell>` in place and borrowing `state` during signal handling or nested builtin execution.
* **Impact:** If `state.borrow_mut()` is called while a previous borrow is still active anywhere on the call stack (e.g. inside a Lua script that calls a builtin that inspects jobs), Rust immediately panics with a fatal crash.
* **Watch Out:** Stick strictly to compile-time `&mut ShellState` references.

### 2. Working Directory Synchronization Desync
* **The Pitfall:** Changing working directory using `std::env::set_current_dir` without updating internal state, or vice versa.
* **Impact:** If the shell updates `state.current_dir` but fails to update the OS process directory (or `$PWD` in `env_vars`), child processes spawned with `fork`/`exec` will start in the wrong directory, and relative path expansion will produce contradictory results.
* **Watch Out:** All directory changes must go through a single method (`state.change_dir(path)`) that updates:
  1. The OS process directory via `std::env::set_current_dir`.
  2. The internal `state.current_dir` cache.
  3. The `PWD` environment variable in `state.env_vars`.

### 3. Logical Operator Short-Circuiting
* **The Pitfall:** Inadvertently executing both sides of an `&&` or `||` branch.
* **Rules to maintain:**
  - `A && B`: Evaluate `A`. If `A` returns an exit code other than `0`, **`B` must never be evaluated or spawned**. Return `A`'s exit code.
  - `A || B`: Evaluate `A`. If `A` returns an exit code of `0`, **`B` must never be evaluated or spawned**. Return `0`.
* **Watch Out:** Background jobs and pipelines inside `A` must finish completely before determining whether to evaluate `B`.

### 4. Exit Status (`$?`) Propagation
* **The Pitfall:** Forgetting to record the exit status of builtins or child processes.
* **Watch Out:** Every single command execution—whether an external binary, a builtin, a pipeline, or a logical chain—must update `state.last_status`. In pipelines (`A | B`), standard POSIX convention sets `$?` to the exit status of the **last command in the pipeline** (unless `pipefail` is implemented).
