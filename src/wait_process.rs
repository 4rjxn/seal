use nix::{
    errno::Errno,
    libc::{STDIN_FILENO, getpgid, getpid, tcsetpgrp},
    sys::wait::{WaitPidFlag, WaitStatus, waitpid},
};

use crate::models::{Job, ShellState};

pub fn wait_for_process(job: Job, state: &mut ShellState) {
    unsafe {
        tcsetpgrp(STDIN_FILENO, job.pgid.into());
    }
    let mut suspended = false;
    for child in &job.childrens {
        loop {
            match waitpid(child.to_owned(), Some(WaitPidFlag::WUNTRACED)) {
                Ok(WaitStatus::Stopped(_, _)) => {
                    println!("stopped {}", &job.command.program);
                    suspended = true;
                    break;
                }
                Ok(_) => break,
                Err(Errno::EINTR) => continue,
                Err(_) => break,
            }
        }
    }
    if suspended {
        state.jobs.push(job);
    }
    unsafe {
        tcsetpgrp(STDIN_FILENO, getpgid(getpid()));
    }
}
