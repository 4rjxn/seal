use std::{cell::RefCell, collections::HashMap, path::PathBuf, rc::Rc};

use nix::unistd::Pid;

use crate::{
    error::ShellResult,
    lua_engine::LuaEngine,
    types::ShellStateType,
    utils::{current_absolute_path, fetch_current_env, get_path_from_env, is_builtin},
};

pub struct Pipeline {
    pub commands: Vec<Command>,
    pub background: bool,
}

pub enum Builtins {
    Echo,
    Lua,
    Exit,
    Type,
    Pwd,
    Cd,
    Fg,
    Exec,
    Jobs,
}

pub struct ShellState {
    pub recent_id: usize,
    pub jobs: Vec<Job>,
    pub env_vars: HashMap<String, String>,
    pub lua_engine: Option<Rc<LuaEngine>>,
}

impl ShellState {
    pub fn new() -> ShellStateType {
        let vars = fetch_current_env();
        let state = Self {
            recent_id: 0,
            jobs: vec![],
            env_vars: vars,
            lua_engine: None,
        };
        let state = Rc::new(RefCell::new(state));
        let mut s = state.borrow_mut();
        s.lua_engine = Some(Rc::new(LuaEngine::new(Rc::clone(&state))));
        Rc::clone(&state)
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
