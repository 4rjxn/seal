use nix::{
    errno::Errno::EINTR,
    libc::{STDIN_FILENO, tcsetpgrp},
    sys::{
        signal::{
            Signal::{SIGCONT, SIGKILL},
            kill, killpg,
        },
        wait::{WaitPidFlag, WaitStatus, waitpid},
    },
    unistd::Pid,
};

use crate::shell::jobs::{JobEvent, JobState, JobTable};

pub fn give_terminal_to(is_interactive: bool, pgid: Pid) {
    if is_interactive {
        unsafe {
            tcsetpgrp(STDIN_FILENO, pgid.as_raw());
        }
    }
}

pub fn reap_background(jobs: &mut JobTable) -> Vec<JobEvent> {
    let flags = WaitPidFlag::WUNTRACED | WaitPidFlag::WNOHANG | WaitPidFlag::WCONTINUED;
    for pgid in jobs.pgids() {
        loop {
            match waitpid(Pid::from_raw(-pgid.as_raw()), Some(flags)) {
                Ok(WaitStatus::StillAlive) => break,
                Ok(st) => {
                    jobs.apply_status(st);
                }
                Err(EINTR) => continue,
                _ => break,
            }
        }
    }

    jobs.take_events()
}

pub struct ForeGroundOutcome {
    pub exit_code: i32,
    pub interrupted: bool,
    pub event: Option<JobEvent>,
}

pub fn wait_foreground(
    jobs: &mut JobTable,
    id: usize,
    resume: bool,
    is_interactive: bool,
    shell_pgid: Pid,
) -> Option<ForeGroundOutcome> {
    let pgid = jobs.get(id)?.pgid;
    jobs.set_background(id, false);
    give_terminal_to(is_interactive, pgid);
    if resume {
        let _ = killpg(pgid, SIGCONT);
        jobs.mark_continued(id);
    }
    while jobs.get(id).map(|j| j.state()) == Some(JobState::Running) {
        match waitpid(Pid::from_raw(-pgid.as_raw()), Some(WaitPidFlag::WUNTRACED)) {
            Ok(st) => {
                jobs.apply_status(st);
            }
            Err(EINTR) => continue,
            Err(_) => break,
        }
    }
    give_terminal_to(is_interactive, shell_pgid);
    let job = jobs.get(id)?;
    let exit_code = job.exit_code();
    let interrupted = job.was_interrupted();

    let event = if job.state() == JobState::Stopped {
        let ev = JobEvent::Stopped {
            id,
            command: job.command.clone(),
        };
        jobs.mark_notified(id);
        jobs.set_background(id, true);
        Some(ev)
    } else {
        jobs.remove(id);
        None
    };

    Some(ForeGroundOutcome {
        exit_code,
        interrupted,
        event,
    })
}

pub fn abort_launch(pgid: Option<Pid>, children: &[Pid]) {
    let group_killed = match pgid {
        Some(g) => killpg(g, SIGKILL).is_ok(),
        None => false,
    };
    if !group_killed {
        for &pid in children {
            let _ = kill(pid, SIGKILL);
        }
    }

    for &pid in children {
        while let Err(EINTR) = waitpid(pid, None) {}
    }
}

//
//resume in background used for 'bg'
//currently its inert
fn _resume_background(jobs: &mut JobTable, id: usize) -> bool {
    let Some(job) = jobs.get(id) else {
        return false;
    };
    let pgid = job.pgid;
    if killpg(pgid, SIGCONT).is_err() {
        return false;
    }
    jobs.mark_continued(id);
    jobs.set_background(id, true);
    true
}
