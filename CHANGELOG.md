# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
