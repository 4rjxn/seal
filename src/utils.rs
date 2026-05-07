use std::{env, path::PathBuf, rc::Rc};

use is_executable::IsExecutable;
use nix::{
    libc::{STDIN_FILENO, getpgrp, tcsetpgrp},
    unistd::Pid,
};

use crate::{
    builtins::job_builtin,
    error::{ShellError, ShellResult},
    models::{Builtins, Job, JobStatus},
    types::ShellStateType,
};

pub fn set_terminal_leader() {
    unsafe {
        let pgid = getpgrp();
        tcsetpgrp(STDIN_FILENO, pgid);
    }
}

pub fn give_terminal_to_job(pid: Pid) {
    unsafe {
        tcsetpgrp(STDIN_FILENO, pid.as_raw());
    }
}

pub fn get_path_from_env(command: &String) -> ShellResult<PathBuf> {
    match env::var_os("PATH") {
        Some(paths) => {
            for mut path in env::split_paths(&paths) {
                path = path.join(command);
                if path.is_executable() {
                    return Ok(path);
                }
            }
            return Err(ShellError::CommandNotFound(command.clone()));
        }
        None => Err(ShellError::CommandNotFound(command.clone())),
    }
}

pub fn is_builtin(path: &str) -> Option<Builtins> {
    match path {
        "echo" => Some(Builtins::Echo),
        "exit" => Some(Builtins::Exit),
        "type" => Some(Builtins::Type),
        "jobs" => Some(Builtins::Jobs),
        "slua" => Some(Builtins::Lua),
        "pwd" => Some(Builtins::Pwd),
        "cd" => Some(Builtins::Cd),
        "fg" => Some(Builtins::Fg),
        _ => None,
    }
}

pub fn current_absolute_path(command: &String) -> ShellResult<PathBuf> {
    let command = command.replace("./", "");
    let curr_dir = env::current_dir().unwrap();
    return Ok(PathBuf::from(curr_dir).join(command));
}

pub fn ok_to_exit(state: ShellStateType) -> bool {
    job_builtin(Rc::clone(&state));
    let s = state.borrow_mut();
    s.jobs.is_empty()
}

pub fn print_job(job: &Job, recent: usize) {
    let status = match job.status {
        JobStatus::Running => "RUNNING",
        JobStatus::Suspended => "SUSPENDED",
        JobStatus::Done => "DONE",
    };
    println!(
        "[{:>2}]{:<1} {:>6} {:<11}{}",
        job.id,
        if job.id == recent { "+" } else { "" },
        job.pgid,
        status,
        job.command.args[0]
    );
}
