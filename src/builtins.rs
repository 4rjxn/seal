use crate::error::{ShellError, ShellResult};
use crate::models::JobStatus;
use crate::types::ShellStateType;
use crate::utils::{get_path_from_env, give_terminal_to_job, ok_to_exit, print_job};
use std::path::PathBuf;
use std::rc::Rc;
use std::{
    env,
    io::{Write, stdout},
    process,
};

use nix::libc::{SIGCONT, killpg};
use nix::sys::wait::{WaitPidFlag, WaitStatus, waitpid};

use crate::{
    models::{Builtins, Command},
    wait_process::wait_for_process,
};

pub fn run_builtin(
    command: &Command,
    state: ShellStateType,
    builtin_type: Builtins,
) -> ShellResult<()> {
    match builtin_type {
        Builtins::Echo => {
            echo_builtin(command)?;
        }
        Builtins::Exit => {
            exit_builtin(state);
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
            fg_builtin(command, state);
        }
        Builtins::Jobs => {
            job_builtin(state);
        }
        Builtins::Lua => {
            lua_builtin(command, state)?;
        }
    }
    Ok(())
}

pub fn job_builtin(state: ShellStateType) {
    let mut state = state.borrow_mut();
    if state.jobs.is_empty() {
        return;
    }
    let recent_id = state.recent_id;
    state.jobs.iter_mut().for_each(|job| {
        match waitpid(job.pgid, Some(WaitPidFlag::WNOHANG)) {
            Ok(WaitStatus::Exited(_, _)) => job.status = JobStatus::Done,
            Ok(_) => {}
            Err(_) => {}
        }
        print_job(&job, recent_id);
    });
    state.jobs.retain(|j| !matches!(j.status, JobStatus::Done));
}

//fn bg_builtin(command: &Command, state: &ShellState) {
//    todo!();
//}

fn fg_builtin(command: &Command, state: ShellStateType) {
    let mut s = state.borrow_mut();
    let job = if let Some(arg) = command.args.get(1) {
        if let Ok(id) = arg.parse::<usize>() {
            s.jobs
                .iter()
                .position(|job| job.id == id)
                .map(|pos| s.jobs.remove(pos))
        } else {
            None
        }
    } else {
        s.jobs.pop()
    };
    match job {
        Some(job) => unsafe {
            give_terminal_to_job(job.pgid);
            killpg(job.pgid.as_raw(), SIGCONT);
            wait_for_process(job, Rc::clone(&state));
            if let Some(job) = s.jobs.last() {
                s.recent_id = job.id
            }
        },
        None => {
            println!("no background process.");
        }
    }
}

fn cd_builtin(command: &Command) -> ShellResult<()> {
    let path: PathBuf;
    match command.args.len() == 1 {
        false => path = PathBuf::from(&command.args[1]),
        true => path = env::home_dir().unwrap_or_else(|| PathBuf::from("/")),
    }
    match env::set_current_dir(path) {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("cd: {}: No such file or directory", command.args[1]);
            Err(e.into())
        }
    }
}

fn lua_builtin(command: &Command, state: ShellStateType) -> ShellResult<()> {
    let script = command.args[1..].join(" ");

    let engine = {
        let state_ref = state.borrow();

        if state_ref.lua_engine.is_none() {
            return Err(ShellError::LuaError);
        }

        state_ref.lua_engine.as_ref().unwrap().clone()
    };

    match engine.run_luastr(&script) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e);
        }
    };
    Ok(())
}

fn exit_builtin(state: ShellStateType) {
    if ok_to_exit(state) {
        process::exit(0)
    } else {
        println!("you have unfinished jobs.");
    }
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
        match get_path_from_env(&command.args[1]) {
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
