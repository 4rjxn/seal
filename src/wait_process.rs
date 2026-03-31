use nix::{
    errno::Errno,
    sys::wait::{waitpid, WaitPidFlag, WaitStatus},
};

use crate::models::{Job, ShellState};

pub fn wait_for_process(job: Job, state: &mut ShellState) {
    loop {
        match waitpid(job.pgid.to_owned(), Some(WaitPidFlag::WUNTRACED)) {
            Ok(WaitStatus::Stopped(_, _)) => {
                println!("stopped {}", job.command.program);
                state.jobs.push(job);
                break;
            }
            Ok(_) => break,
            Err(Errno::EINTR) => continue,
            Err(_) => break,
        }
    }
}
