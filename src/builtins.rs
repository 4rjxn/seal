use crate::parser::{is_builtin, locate_command};
use std::{
    env,
    io::{Write, stdout},
    path::PathBuf,
    process,
};

use nix::libc::{SIGCONT, STDIN_FILENO, getpgid, getpid, kill, tcsetpgrp};

use crate::{
    models::{Builtins, Command, ShellState},
    wait_process::wait_for_process,
};

pub fn run_builtin(command: &Command, state: &mut ShellState, builtin_type: Builtins) {
    match builtin_type {
        Builtins::Echo => {
            echo_builtin(command);
        }
        Builtins::Exit => {
            exit_builtin();
        }
        Builtins::Type => {
            type_builtin(command);
        }
        Builtins::Pwd => {
            pwd_builtin();
        }
        Builtins::Cd => {
            cd_builtin(command);
        }
        Builtins::Fg => {
            fg_builtin(state);
        }
    }
}

fn fg_builtin(state: &mut ShellState) {
    match state.jobs.pop() {
        Some(job) => unsafe {
            tcsetpgrp(STDIN_FILENO, job.pgid.as_raw());
            kill(job.pgid.as_raw(), SIGCONT);
            wait_for_process(job, state);
            tcsetpgrp(STDIN_FILENO, getpgid(getpid()));
        },
        None => return,
    }
}

fn cd_builtin(command: &Command) {
    let mut path = PathBuf::from(env::home_dir().unwrap());
    if command.args.len() > 1 {
        let abspath = &command.args[1].replacen("~", env::home_dir().unwrap().to_str().unwrap(), 1);
        path = PathBuf::from(abspath);
    }
    match env::set_current_dir(&path) {
        Ok(_) => (),
        Err(_) => println!("cd: {}: No such file or directory", path.to_str().unwrap()),
    }
}

fn exit_builtin() {
    process::exit(0)
}

fn echo_builtin(command: &Command) {
    let data_string = command.args.join(" ") + "\n";
    stdout().write_all(data_string.as_bytes()).unwrap();
}

fn type_builtin(command: &Command) {
    if command.args.len() > 1 {
        if is_builtin(&command.args[1]).is_ok() {
            stdout()
                .write_all(format!("{} is a shell builtin\n", command.args[1]).as_bytes())
                .unwrap();
            return;
        }
        match locate_command(&command.args[1]) {
            Ok(path) => {
                stdout()
                    .write_all(
                        format!("{} is {}\n", &command.args[1], path.to_str().unwrap()).as_bytes(),
                    )
                    .unwrap();
                return;
            }
            Err(_) => (),
        }
        stdout()
            .write_all(format!("{}: not found\n", command.args[1]).as_bytes())
            .unwrap();
    }
}

fn pwd_builtin() {
    let path = env::current_dir().unwrap();
    println!("{}", path.to_str().unwrap())
}
