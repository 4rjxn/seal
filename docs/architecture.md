 # The Architecture: "Shell as an Engine"

    ┌──────────────────────────────────────────────────────────────┐
    │                      FRONTENDS                               │
    │                                                              │
    │   Terminal REPL (Interactive)       Lua Engine (Scripting)   │
    │   (Rustyline, Prompt, History)      (seal.run, seal.cd, ...) │
    │                │                                │            │
    └────────────────┼────────────────────────────────┼────────────┘
                     │                                │
                     ▼                                ▼
    ┌────────────────────────────────────────────────────────┐
    │                   SEAL CORE API (lib.rs)               │
    │                                                        │
    │   • execute(ast, &mut state)                           │
    │   • execute_str(input, &mut state)                     │
    │   • execute_command(cmd, args, &mut state)             │
    │   • state.change_dir(path)                             │
    │   • state.get_env() / state.set_env()                  │
    │                                                        │
    │   [ Lexer ] ───▶ [ Parser ] ───▶ [ Execution Engine ]  │
    │   [ Redirections ]  [ Job Control ]  [ Builtins ]      │
    └────────────────────────────────────────────────────────┘


This effectively decouples the shell from the frontends. As a result the
codebase will become more maintainable. This is the major shift iam planning,
before this there are some more small steps which i need to take. which includes
improving current parser and lexer, and transition to a library based project.

