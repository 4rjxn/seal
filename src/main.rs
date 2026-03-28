mod builtins;
mod parser;

use nix::{
    errno::Errno,
    fcntl::{OFlag, open},
    libc::{STDIN_FILENO, STDOUT_FILENO, dup2, getpgid, getpid, tcsetpgrp},
    sys::{
        signal::{SigHandler, Signal, signal},
        stat::Mode,
        wait::waitpid,
    },
    unistd::{Pid, close, execvp, fork, pipe, setpgid},
};

use crate::{
    builtins::run_builtin,
    parser::{Command, Redirects, parse_pipelines, read_and_parse},
};
use std::{
    ffi::CString,
    io::{Write, stdout},
    os::fd::{AsRawFd, OwnedFd},
    process,
};

fn set_redirection(command: &Command) {
    if command.redirects.is_empty() {
        return;
    }
    for redirect in &command.redirects {
        match redirect {
            Redirects::Output(file) => {
                let fd = open(
                    file.as_str(),
                    OFlag::O_CREAT | OFlag::O_WRONLY | OFlag::O_TRUNC,
                    Mode::from_bits(0o644).unwrap(),
                )
                .expect("open failed");
                unsafe { dup2(fd.as_raw_fd(), 1) };
            }
            Redirects::OutputErr(file) => {
                let fd = open(
                    file.as_str(),
                    OFlag::O_CREAT | OFlag::O_WRONLY | OFlag::O_TRUNC,
                    Mode::from_bits(0o644).unwrap(),
                )
                .expect("open failed");
                unsafe { dup2(fd.as_raw_fd(), 2) };
            }
            Redirects::Append(file) => {
                let fd = open(
                    file.as_str(),
                    OFlag::O_CREAT | OFlag::O_WRONLY | OFlag::O_APPEND,
                    Mode::from_bits(0o644).unwrap(),
                )
                .expect("open failed");
                unsafe { dup2(fd.as_raw_fd(), 1) };
            }
            Redirects::AppendErr(file) => {
                let fd = open(
                    file.as_str(),
                    OFlag::O_CREAT | OFlag::O_WRONLY | OFlag::O_APPEND,
                    Mode::from_bits(0o644).unwrap(),
                )
                .expect("open failed");
                unsafe { dup2(fd.as_raw_fd(), 2) };
            }
        }
    }
}

fn execute_command(commands: &Vec<Command>) {
    let mut prev_read: Option<OwnedFd> = None;
    if commands.len() == 1 {
        if let Ok(built_in) = commands[0].is_builtin() {
            run_builtin(&commands[0], built_in);
        }
    }
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
                        setpgid(Pid::from_raw(0), Pid::from_raw(0)).unwrap();
                        let c = CString::new(command.program.as_bytes()).unwrap();
                        let mut cargs = Vec::new();
                        for arg in &command.args {
                            cargs.push(CString::new(arg.as_bytes()).unwrap());
                        }
                        if let Some(fd) = &prev_read {
                            dup2(fd.as_raw_fd(), STDIN_FILENO);
                        }
                        if let Some(w) = write_end {
                            dup2(w.as_raw_fd(), STDOUT_FILENO);
                        }

                        set_redirection(&command);
                        if let Ok(built_in) = command.is_builtin() {
                            run_builtin(&command, built_in);
                            process::exit(0);
                        } else {
                            let _ = execvp(&c, &cargs);
                        }
                    }
                    nix::unistd::ForkResult::Parent { child } => {
                        if let Some(fd) = prev_read {
                            close(fd).unwrap();
                        }
                        if let Some(w) = write_end {
                            close(w).unwrap();
                        }
                        prev_read = read_end;
                        setpgid(child, child).unwrap();
                        tcsetpgrp(STDIN_FILENO, child.into());
                        loop {
                            match waitpid(child, None) {
                                Ok(_) => break,
                                Err(Errno::EINTR) => continue,
                                Err(_) => break,
                            }
                        }
                        tcsetpgrp(STDIN_FILENO, getpgid(getpid()));
                    }
                }
            }
        }
    }
}

fn process_command(commands: Vec<Command>) -> Result<(), ()> {
    for command in &commands {
        if !command.is_valid() {
            stdout()
                .write_all(format!("{}: command not found\n", &command.program).as_bytes())
                .unwrap();
            return Err(());
        }
    }
    execute_command(&commands);
    Ok(())
}

fn main() {
    loop {
        let command = read_and_parse();
        let pipeline = parse_pipelines(command).unwrap();
        let _ = process_command(pipeline.commands);
    }
}
