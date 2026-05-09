# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-05-09

### Added
- **Lua Engine**: Integrated `mlua` to support Lua scripting. Added `slua` builtin to run Lua scripts and one-liners.
- **Lua API**: Added a `read()` API for Lua scripts.
- **Job Control**: Added background task management (`&`), `jobs` builtin, and basic job listing.
- **Built-in Commands**: Added `exec` builtin for replacing the shell process.
- **Globbing & Expansion**: Added basic globbing support and improved path expansion (including `~` and `./`).
- **File Completion**: Basic filename completion for improved user experience.
- **Output Capturing**: Added capability to capture command output.
- **History Integration**: Command history persistence and navigation.
- **Improved Prompt**: Dynamic prompt showing user, hostname, and current working directory.
- **Installation Script**: Added `invoke.sh` for easy installation.

### Changed
- **Architectural Refactor**: Completely modularized the codebase into separate modules (`lexer`, `parser`, `execution`, `redirection`, `repl`, etc.), significantly improving maintainability and scalability.
- **Enhanced Parsing**: Implemented a more robust lexer and parser with better error handling.
- **Piping & Redirection**: Improved robustness of pipelines and I/O redirection, including support for piping between built-ins and external commands.

### Fixed
- Fixed a random blank terminal issue.
- Resolved redirection bugs specifically affecting built-in commands.
- Fixed handling of spaces in file paths.
- Improved signal handling and job control guarding.
- Various minor bug fixes and general error handling improvements.

## [0.1.0] - 2026-03-29

### Added
- **Core Shell Infrastructure**: Initial implementation of the Seal shell environment.
- **Process Management**:
    - Support for executing external programs using `fork` and `execvp`.
    - Signal handling for `SIGINT` (Ctrl+C) and `SIGTTOU`.
    - Terminal process group management to handle foreground and background switching.
- **Built-in Commands**:
    - `cd`: Change directory with `~` expansion.
    - `echo`: Print text with support for arguments and redirection.
    - `pwd`: Display the current working directory.
    - `type`: Check if a command is a shell builtin or an external binary.
    - `exit`: Cleanly terminate the shell session.
- **Advanced Parsing**:
    - Tokenizer with support for single quotes (`'`), double quotes (`"`), and backslash (`\`) escapes.
    - Support for piping (`|`) between multiple commands.
- **I/O Redirection**:
    - Standard output redirection (`>` and `>>`).
    - Standard error redirection (`2>` and `2>>`).
- **User Experience**:
    - Integrated `rustyline` for command-line editing and history persistence.
    - Automatic path resolution for binaries found in the `$PATH` environment variable.
