use std::path::PathBuf;

use nix::unistd::Pid;

use crate::{
    error::ShellResult,
    utils::{current_absolute_path, get_path_from_env, is_builtin},
};

pub struct Pipeline {
    pub commands: Vec<Command>,
    pub background: bool,
}

pub enum Builtins {
    Echo,
    Exit,
    Type,
    Pwd,
    Cd,
    Fg,
    Jobs,
}

pub struct ShellState {
    pub recent_id: usize,
    pub jobs: Vec<Job>,
}

impl ShellState {
    pub fn new() -> Self {
        Self {
            recent_id: 0,
            jobs: vec![],
        }
    }
}

#[derive(Clone)]
pub enum JobStatus {
    Running,
    Suspended,
    Done,
}

#[derive(Clone)]
pub struct Job {
    pub id: usize,
    pub pgid: Pid,
    pub status: JobStatus,
    pub command: Command,
    pub childrens: Vec<Pid>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LexerState {
    Normal,
    InSingleQuote,
    InDoubleQuote,
    Escaped { return_to: Box<LexerState> },
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Word { value: String, quoted: bool },
    Pipe,
    Output,
    Append,
    OutputErr,
    AppendErr,
    Background,
}

#[derive(Debug, Clone)]
pub struct Redirect {
    pub kind: RedirectKind,
    pub target: String,
}

#[derive(Debug, Clone)]
pub enum RedirectKind {
    Output,
    OutputErr,
    Append,
    AppendErr,
}

#[derive(Debug, Clone)]
pub struct Command {
    pub program: String,
    pub args: Vec<String>,
    pub redirects: Vec<Redirect>,
}

impl Command {
    pub fn kind(&self) -> CommandKind {
        if let Some(builtin) = is_builtin(&self.program) {
            return CommandKind::Builtin(builtin);
        }
        CommandKind::External
    }
    pub fn find_binary_from_path(&mut self) -> bool {
        if let Ok(path) = self.locate_command() {
            self.program = path.to_str().unwrap().to_string();
            return true;
        }
        false
    }
    fn locate_command(&self) -> ShellResult<PathBuf> {
        if self.program.starts_with("./") {
            return current_absolute_path(&self.program);
        }
        get_path_from_env(&self.program)
    }
}

pub enum CommandKind {
    Builtin(Builtins),
    External,
}
