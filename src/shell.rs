use nix::unistd::{Pid, getpgrp};

use crate::{
    lexer,
    parser::{self, node::SyntaxNode},
    shell::{env::Env, jobs::JobTable},
};

pub mod env;
pub mod executor;
pub mod jobs;
pub mod shell_error;

pub struct Shell {
    pub env: Env,
    pub jobs: JobTable,
    pub last_exit: i32,
    pub shell_pgid: Pid,
}

impl Shell {
    pub fn new() -> Self {
        Self {
            env: Env::new(),
            jobs: JobTable::new(),
            last_exit: 0,
            shell_pgid: getpgrp(),
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
