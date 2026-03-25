use std::{
    env,
    io::{Write, stderr, stdin, stdout},
    path::PathBuf,
    process::{self, Command},
};

use is_executable::IsExecutable;

fn read_and_parse() -> Vec<String> {
    let mut command_input = String::new();
    print!("$ ");
    stdout().flush().unwrap();
    stdin().read_line(&mut command_input).unwrap();
    let parsed_command: Vec<String> = command_input
        .split_whitespace()
        .map(|s| s.to_owned())
        .collect();
    parsed_command
}

fn is_valid(command: &[String]) -> bool {
    is_builtin(&command[0]) || is_valid_command(&command[0])
}

fn locate_command(command: &String) -> Result<PathBuf, ()> {
    match env::var_os("PATH") {
        Some(paths) => {
            for mut path in env::split_paths(&paths) {
                path = path.join(command);
                if path.is_executable() {
                    return Ok(path);
                }
            }
            return Err(());
        }
        None => Err(()),
    }
}

fn is_valid_command(command: &String) -> bool {
    match locate_command(command) {
        Ok(_) => return true,
        Err(_) => return false,
    }
}

fn is_builtin(command: &String) -> bool {
    return match command.as_str() {
        "exit" => true,
        "echo" => true,
        "type" => true,
        "pwd" => true,
        "cd" => true,
        _ => false,
    };
}

fn cd_builtin(command: &[String]) {
    let mut path = PathBuf::from(env::home_dir().unwrap());
    if command.len() != 0 {
        let abspath = &command[0].replacen("~", env::home_dir().unwrap().to_str().unwrap(), 1);
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

fn echo_builtin(args: &[String]) {
    print!("{}\n", args.join(" "));
}

fn type_builtin(command: &[String]) {
    if command.len() > 1 {
        if is_builtin(&command[1]) {
            println!("{} is a shell builtin", command[1]);
            return;
        }
        match locate_command(&command[1]) {
            Ok(path) => {
                println!("{} is {}", &command[1], path.to_str().unwrap());
                return;
            }
            Err(_) => (),
        }
        println!("{}: not found", command[1]);
    }
}

fn pwd_builtin() {
    let path = env::current_dir().unwrap();
    println!("{}", path.to_str().unwrap())
}

fn execute_command(command: &[String]) {
    let path = locate_command(&command[0]).unwrap();
    let mut process = Command::new(&path);
    if command.len() > 1 {
        process.args(&command[1..]);
    }
    let status = process.output().unwrap();
    stdout().write_all(&status.stdout).unwrap();
    stderr().write_all(&status.stderr).unwrap();
}

fn process_command(command: &[String]) -> Result<(), ()> {
    if command[0] == "exit" {
        exit_builtin()
    }
    match command[0].as_str() {
        "echo" => Ok(echo_builtin(&command[1..])),
        "type" => Ok(type_builtin(&command)),
        "cd" => Ok(cd_builtin(&command[1..])),
        "pwd" => Ok(pwd_builtin()),
        _ => Ok(execute_command(command)),
    }
}

fn main() {
    loop {
        let command = read_and_parse();
        if is_valid(&command) {
            match process_command(&command) {
                Ok(_) => continue,
                Err(_) => (),
            }
        }
        print!("{}: command not found\n", command[0]);
    }
}
