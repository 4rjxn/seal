use nix::unistd::Pid;

use crate::shell::{env::Env, jobs::JobTable};

mod env;
mod executor;
mod jobs;

struct Shell {
    env: Env,
    jobs: JobTable,
    last_exit: i32,
    shell_pgid: Pid,
}

impl Shell {
    fn new() -> Self {
        Self {
            env: Env::new(),
            jobs: JobTable::new(),
            last_exit: 0,
            shell_pgid: Pid::this(),
        }
    }

    fn execute(&mut self) -> i32 {
        todo!()
    }
}
