use crate::error::ShellResult;
use crate::parser::is_valid_command;
use nix::unistd::Pid;

use crate::parser::is_builtin;

pub struct Pipeline {
    pub commands: Vec<Command>,
}

pub enum Builtins {
    Echo,
    Exit,
    Type,
    Pwd,
    Cd,
    Fg,
}

pub struct ShellState {
    pub jobs: Vec<Job>,
}

pub struct Job {
    pub pgid: Pid,
    pub command: Command,
}

#[derive(Debug)]
pub enum Tokens {
    Word(String),
    Pipe,
    Output,
    Append,
    OutputErr,
    AppendErr,
}

#[derive(Debug, Clone)]
pub enum Redirects {
    //Input(String),
    Output(String),
    OutputErr(String),
    Append(String),
    AppendErr(String),
}

#[derive(Debug, Clone)]
pub struct Command {
    pub program: String,
    pub args: Vec<String>,
    pub redirects: Vec<Redirects>,
}

impl Command {
    pub fn is_valid(&self) -> bool {
        is_builtin(&self.program).is_ok() || is_valid_command(&self.program)
    }
    pub fn is_builtin(&self) -> ShellResult<Builtins> {
        is_builtin(&self.program)
    }
}
