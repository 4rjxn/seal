use crate::error::ShellResult;
use crate::models::JobStatus;
use crate::parser::{is_builtin, locate_command};
use crate::utils::print_job;
use std::{
    env,
    io::{Write, stdout},
    path::PathBuf,
    process,
};

use nix::libc::{SIGCONT, killpg};
use nix::sys::wait::{WaitPidFlag, WaitStatus, waitpid};

use crate::{
    models::{Builtins, Command, ShellState},
    wait_process::wait_for_process,
};

pub fn run_builtin(
    command: &Command,
    state: &mut ShellState,
    builtin_type: Builtins,
) -> ShellResult<()> {
    match builtin_type {
        Builtins::Echo => {
            echo_builtin(command)?;
        }
        Builtins::Exit => {
            exit_builtin();
        }
        Builtins::Type => {
            type_builtin(command)?;
        }
        Builtins::Pwd => {
            pwd_builtin()?;
        }
        Builtins::Cd => {
            cd_builtin(command)?;
        }
        Builtins::Fg => {
            fg_builtin(state);
        }
        Builtins::Jobs => {
            job_builtin(state);
        }
    }
    Ok(())
}

fn job_builtin(state: &mut ShellState) {
    if state.jobs.is_empty() {
        return;
    }
    for job in &mut state.jobs {
        match waitpid(job.pgid, Some(WaitPidFlag::WNOHANG)) {
            Ok(WaitStatus::Exited(_, _)) => job.status = JobStatus::Done,
            Ok(_) => {}
            Err(_) => {}
        }
        print_job(job, state.recent_id);
    }
    state.jobs.retain(|j| !matches!(j.status, JobStatus::Done));
}

fn fg_builtin(state: &mut ShellState) {
    match state.jobs.pop() {
        Some(job) => unsafe {
            killpg(job.pgid.as_raw(), SIGCONT);
            wait_for_process(job, state);
            if let Some(job) = state.jobs.last() {
                state.recent_id = job.id
            }
        },
        None => {
            println!("no background process.");
        }
    }
}

fn cd_builtin(command: &Command) -> ShellResult<()> {
    let home = env::var("HOME").unwrap_or_else(|_| "/".to_string());
    let mut path = PathBuf::from(&home);
    if command.args.len() > 1 {
        let abspath = &command.args[1].replacen("~", &home, 1);
        path = PathBuf::from(abspath);
    }
    match env::set_current_dir(&path) {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("cd: {}: No such file or directory", path.to_str().unwrap());
            Err(e.into())
        }
    }
}

fn exit_builtin() {
    process::exit(0)
}

fn echo_builtin(command: &Command) -> ShellResult<()> {
    let mut args = command.args.clone();
    if !args.is_empty() {
        args.remove(0); // remove "echo"
    }
    let data_string = args.join(" ") + "\n";
    stdout().write_all(data_string.as_bytes())?;
    Ok(())
}

fn type_builtin(command: &Command) -> ShellResult<()> {
    if command.args.len() > 1 {
        if is_builtin(&command.args[1]).is_ok() {
            stdout().write_all(format!("{} is a shell builtin\n", command.args[1]).as_bytes())?;
            return Ok(());
        }
        match locate_command(&command.args[1]) {
            Ok(path) => {
                stdout().write_all(
                    format!("{} is {}\n", &command.args[1], path.to_str().unwrap()).as_bytes(),
                )?;
                return Ok(());
            }
            Err(_) => (),
        }
        stdout().write_all(format!("{}: not found\n", command.args[1]).as_bytes())?;
    }
    Ok(())
}

fn pwd_builtin() -> ShellResult<()> {
    let path = env::current_dir()?;
    println!("{}", path.to_str().unwrap());
    Ok(())
}
