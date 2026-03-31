use nix::{
    libc::{dup2, getpgid, getpid, tcsetpgrp, STDOUT_FILENO},
    sys::signal::{signal, SigHandler, Signal},
    unistd::{close, execvp, fork},
};
use std::{
    ffi::CString,
    os::fd::{AsRawFd, OwnedFd},
    process,
};

use nix::{
    libc::STDIN_FILENO,
    unistd::{pipe, setpgid, Pid},
};

use crate::{
    builtins::run_builtin,
    models::{Command, Job, ShellState},
    redirection::set_redirection,
    wait_process::wait_for_process,
};

pub fn execute_command(commands: &Vec<Command>, state: &mut ShellState) {
    let mut prev_read: Option<OwnedFd> = None;
    if commands.len() == 1 {
        if let Ok(built_in) = commands[0].is_builtin() {
            let _ = run_builtin(&commands[0], state, built_in);
            return;
        }
    }
    let mut children = Vec::new();
    let mut pgid: Option<Pid> = None;
    for (i, command) in commands.iter().enumerate() {
        let (read_end, write_end) = if i < commands.len() - 1 {
            let (r, w) = pipe().unwrap();
            (Some(r), Some(w))
        } else {
            (None, None)
        };
        unsafe {
            signal(Signal::SIGTTOU, SigHandler::SigIgn).unwrap();
            if let Ok(fork_result) = fork() {
                match fork_result {
                    nix::unistd::ForkResult::Child => {
                        signal(Signal::SIGINT, SigHandler::SigDfl).unwrap();
                        signal(Signal::SIGTSTP, SigHandler::SigDfl).unwrap();
                        if let Some(pgid) = pgid {
                            setpgid(Pid::from_raw(0), pgid).unwrap();
                        } else {
                            setpgid(Pid::from_raw(0), Pid::from_raw(0)).unwrap();
                        }
                        let c = CString::new(command.program.as_bytes()).unwrap();
                        let mut cargs = Vec::new();
                        for arg in &command.args {
                            cargs.push(CString::new(arg.as_bytes()).unwrap());
                        }
                        if let Some(r) = &prev_read {
                            dup2(r.as_raw_fd(), STDIN_FILENO);
                            close(r.as_raw_fd()).unwrap();
                        }
                        if let Some(w) = write_end {
                            dup2(w.as_raw_fd(), STDOUT_FILENO);
                            close(w).unwrap();
                        }
                        if let Some(r) = read_end {
                            close(r).unwrap();
                        }
                        if let Err(e) = set_redirection(&command) {
                            eprintln!("redirection error: {}", e);
                            process::exit(1);
                        }
                        if let Ok(built_in) = command.is_builtin() {
                            let _ = run_builtin(&command, state, built_in);
                            process::exit(0);
                        } else {
                            let _ = execvp(&c, &cargs);
                        }
                    }
                    nix::unistd::ForkResult::Parent { child } => {
                        if pgid.is_none() {
                            pgid = Some(child);
                        }
                        if let Some(r) = prev_read {
                            close(r).unwrap();
                        }
                        if let Some(w) = write_end {
                            close(w).unwrap();
                        }
                        prev_read = read_end;
                        children.push(child);
                    }
                }
            }
        }
    }
    unsafe {
        tcsetpgrp(STDIN_FILENO, pgid.unwrap().into());
    }
    let job = Job {
        pgid: Pid::from_raw(pgid.unwrap().into()),
        command: commands.last().unwrap().clone(),
    };
    wait_for_process(job, state);
    unsafe {
        tcsetpgrp(STDIN_FILENO, getpgid(getpid()));
    }
}
