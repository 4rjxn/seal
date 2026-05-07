use nix::{
    errno::Errno,
    sys::wait::{WaitPidFlag, WaitStatus, waitpid},
};

use crate::{
    models::{Job, JobStatus},
    types::ShellStateType,
    utils::{print_job, set_terminal_leader},
};

pub fn wait_for_process(mut job: Job, state: ShellStateType) {
    let mut state = state.borrow_mut();
    let mut suspended = false;
    for child in &job.childrens {
        loop {
            match waitpid(child.to_owned(), Some(WaitPidFlag::WUNTRACED)) {
                Ok(WaitStatus::Stopped(_, _)) => {
                    job.status = JobStatus::Suspended;
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
        state.recent_id = job.id;
        print_job(&job, state.recent_id);
        state.jobs.push(job);
    }
    set_terminal_leader();
}
