mod parser;

use nix::{
    errno::Errno,
    fcntl::{OFlag, open},
    libc::{STDIN_FILENO, dup2, getpgid, getpid, tcsetpgrp},
    sys::{
        signal::{SigHandler, Signal, signal},
        stat::Mode,
        wait::waitpid,
    },
    unistd::{Pid, execvp, fork, setpgid},
};

use crate::parser::{
    Command, Redirects, is_builtin, locate_command, parse_pipelines, read_and_parse,
};
use std::{
    env,
    ffi::CString,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Cursor, Read, Write},
    os::fd::AsRawFd,
    path::PathBuf,
    process,
};

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

fn echo_builtin(command: &Command) -> Option<Cursor<String>> {
    println!("santhue {:?}", &command.args);
    let data_string = command.args.join(" ") + "\n";
    if command.redirects.len() > 0 {
        match &command.redirects[0] {
            Redirects::Output(file) => {
                let mut file = File::create(file).unwrap();
                file.write_all(data_string.as_bytes()).unwrap();
                return None;
            }
            Redirects::OutputErr(file) => {
                let _ = File::create(file).unwrap();
            }
            Redirects::Append(file) => {
                let mut file = OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(file)
                    .unwrap();
                file.write_all(data_string.as_bytes()).unwrap();
                return None;
            }
            Redirects::AppendErr(_) => {}
        }
    }
    return Some(Cursor::new(data_string));
}

fn type_builtin(command: &Command) {
    if command.args.len() > 1 {
        if is_builtin(&command.args[1]) {
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

fn execute_command<R: Read>(command: &Command, _data: &mut Option<R>) {
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
                    set_redirection(&command);
                    execvp(&c, &cargs).expect("baaaaaaaad");
                }
                nix::unistd::ForkResult::Parent { child } => {
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

fn process_command(commands: Vec<Command>) -> Result<(), ()> {
    let mut data_buff: Option<Box<dyn Read>> = None;
    for command in commands {
        if !command.is_valid() {
            println!("{}: command not found", command.program);
            return Err(());
        }
        if command.program == "exit" {
            exit_builtin()
        }
        match command.program.as_str() {
            "echo" => {
                if let Some(data) = echo_builtin(&command) {
                    data_buff = Some(Box::from(data));
                }
            }
            "type" => type_builtin(&command),
            "cd" => cd_builtin(&command),
            "pwd" => pwd_builtin(),
            _ => execute_command(&command, &mut data_buff),
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
