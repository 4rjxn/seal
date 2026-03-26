mod parser;

use crate::parser::{
    Command, Redirects, is_builtin, locate_command, parse_pipelines, read_and_parse,
};
use std::{
    env,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write, stdout},
    path::PathBuf,
    process::{self, ChildStdout, Command as OsCommand, Stdio},
};

fn cd_builtin(command: &Command) {
    let mut path = PathBuf::from(env::home_dir().unwrap());
    if command.args.len() != 0 {
        let abspath = &command.args[0].replacen("~", env::home_dir().unwrap().to_str().unwrap(), 1);
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
    if command.redirects.len() > 0 {
        match &command.redirects[0] {
            Redirects::Output(file) => {
                let mut file = File::create(file).unwrap();
                file.write_all(data_string.as_bytes()).unwrap();
            }
            Redirects::Append(file) => {
                let mut file = OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(file)
                    .unwrap();
                file.write_all(data_string.as_bytes()).unwrap();
            }
        }
        return;
    }
    stdout().write_all(data_string.as_bytes()).unwrap();
}

fn type_builtin(command: &Command) {
    if command.args.len() > 0 {
        if is_builtin(&command.args[0]) {
            println!("{} is a shell builtin", command.args[0]);
            return;
        }
        match locate_command(&command.args[0]) {
            Ok(path) => {
                println!("{} is {}", &command.args[0], path.to_str().unwrap());
                return;
            }
            Err(_) => (),
        }
        println!("{}: not found", command.args[0]);
    }
}

fn pwd_builtin() {
    let path = env::current_dir().unwrap();
    println!("{}", path.to_str().unwrap())
}

fn execute_command(command: &Command, data: Option<ChildStdout>) -> Option<ChildStdout> {
    let mut process = OsCommand::new(&command.program);
    process.args(&command.args);
    match data {
        Some(data) => {
            process.stdin(Stdio::from(data));
        }
        None => (),
    }
    if command.redirects.len() > 0 {
        match &command.redirects[0] {
            Redirects::Output(file) => {
                let f = File::create(file).unwrap();
                process.stdout(f);
            }
            Redirects::Append(file) => {
                let f = OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(file)
                    .unwrap();
                process.stdout(f);
            }
        }
    } else {
        process.stdout(Stdio::piped()).stderr(Stdio::inherit());
    }
    let mut child = process.spawn().unwrap();
    child.wait().unwrap();
    Some(child.stdout.unwrap())
}

fn process_command(commands: Vec<Command>) -> Result<(), ()> {
    let mut data_buff: Option<ChildStdout> = None;
    for command in commands {
        if !command.is_valid() {
            println!("{}: command not found", command.program);
            return Err(());
        }
        if command.program == "exit" {
            exit_builtin()
        }
        match command.program.as_str() {
            "echo" => echo_builtin(&command),
            "type" => type_builtin(&command),
            "cd" => cd_builtin(&command),
            "pwd" => pwd_builtin(),
            _ => {
                data_buff = execute_command(&command, data_buff);
            }
        }
    }
    match data_buff {
        Some(data) => {
            let mut reader = BufReader::new(data);
            let mut line = String::new();

            while reader.read_line(&mut line).expect("failed to read line") > 0 {
                print!("{}", line); // Print the line as it's read
                line.clear(); // Clear the buffer for the next line
            }
        }
        None => {}
    }
    Ok(())
}

fn main() {
    loop {
        let command = read_and_parse();
        let pipeline = parse_pipelines(command).unwrap();
        let _ = process_command(pipeline.commands);
    }
}
