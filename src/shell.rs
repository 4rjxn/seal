use nix::unistd::{Pid, getpgrp};

use crate::{
    lexer,
    parser::{self, node::SyntaxNode},
    shell::{builtins::BuiltinRegistry, env::Env, jobs::JobTable, opts::ShellOpts},
};

pub mod builtins;
pub mod env;
pub mod executor;
pub mod job_control;
pub mod jobs;
pub mod opts;

pub struct Shell {
    pub env: Env,
    pub jobs: JobTable,
    pub last_exit: i32,
    pub shell_pgid: Pid,
    pub builtins: BuiltinRegistry,
    pub shell_opts: ShellOpts,
}

impl Shell {
    pub fn new() -> Self {
        Self::with_opts(ShellOpts::default())
    }

    pub fn with_opts(opts: ShellOpts) -> Self {
        Self {
            env: Env::new(),
            jobs: JobTable::new(),
            last_exit: 0,
            shell_pgid: getpgrp(),
            builtins: BuiltinRegistry::with_default(),
            shell_opts: opts,
        }
    }

    pub fn execute(&mut self, syntax: &SyntaxNode) -> i32 {
        let exit_code = executor::execute(self, syntax);
        self.last_exit = exit_code;
        exit_code
    }

    pub fn execute_str(&mut self, str: &str) -> i32 {
        let tokens = match lexer::tokenise(str) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("{e}");
                return 2;
            }
        };
        let syntax = match parser::parse(tokens) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("{e}");
                return 2;
            }
        };
        self.execute(&syntax)
    }
}

impl Default for Shell {
    fn default() -> Self {
        Self::new()
    }
}
