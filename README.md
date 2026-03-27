# Seal 🦭

Seal is a lightweight, Unix-like shell written in Rust. It provides a familiar command-line interface with support for essential shell features, including built-in commands, external program execution, input/output redirection, and piping. Seal is my personal thing as result this may contain some explicit content.

## Features

- **Built-in Commands**:
  - `cd [dir]`: Change the current working directory. Supports `~` for the home directory.
  - `echo [args]`: Print arguments to the standard output.
  - `pwd`: Print the current working directory.
  - `type [command]`: Indicate how a command would be interpreted (builtin or external).
  - `exit`: Exit the shell.
- **External Command Execution**: Seamlessly runs programs available in your `$PATH`.
- **Redirection**:
  - `>` or `1>`: Redirect standard output to a file (truncates).
  - `2>`: Redirect standard error to a file (truncates).
  - `>>` or `1>>`: Append standard output to a file.
  - `2>>`: Append standard error to a file.
- **Pipelining**: Support for basic command piping (`cmd1 | cmd2`).
- **Quote Handling**: Supports single quotes (`'`), double quotes (`"`), and backslashes (`\`) for complex command arguments.
- **Line Editing**: Powered by `rustyline` for a smooth terminal experience with history and line editing capabilities.

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (2024 edition or later)

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/seal.git
   cd seal
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

3. Run the shell:
   ```bash
   ./target/release/seal
   ```

## Usage Examples

### Basic Commands
```bash
>> pwd
/home/user/seal
>> echo Hello, Seal!
Hello, Seal!
```

### Redirection
```bash
>> echo "Hello, World" > hello.txt
>> cat hello.txt
Hello, World
>> ls non_existent_file 2> error.log
```

### Pipelines
```bash
>> ls | grep .rs
main.rs
parser.rs
```

### Complex Arguments
```bash
>> echo "This is a \"quoted\" argument"
This is a "quoted" argument
```

## Dependencies

- `rustyline`: For command-line editing and history.
- `nix`: For Unix system calls (fork, exec, signals).
- `is_executable`: To verify executable permissions in `$PATH`.

## License

This project is open-source and available under the GNU General Public License version 2.0 (GPL-2.0).
