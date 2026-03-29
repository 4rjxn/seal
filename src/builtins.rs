use std::{
    env,
    io::{Write, stdout},
    path::PathBuf,
    process,
};

use crate::parser::{Builtins, Command, is_builtin, locate_command};

pub fn run_builtin(command: &Command, builtin_type: Builtins) {
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
            println!("{} is a shell builtin", command.args[1]);
            return;
        }
        match locate_command(&command.args[1]) {
            Ok(path) => {
                println!("{} is {}", &command.args[1], path.to_str().unwrap());
                return;
            }
            Err(_) => (),
        }
        println!("{}: not found", command.args[1]);
    }
}

fn pwd_builtin() {
    let path = env::current_dir().unwrap();
    println!("{}", path.to_str().unwrap())
}
