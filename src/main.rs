mod builtins;
mod execution;
mod models;
mod parser;
mod prompt;
mod redirection;
mod wait_process;

use nix::sys::signal::{SigHandler, Signal, signal};

use crate::{
    execution::execute_command,
    models::{Command, ShellState},
    parser::{parse_pipelines, read_and_parse},
};
use std::io::{Write, stdout};

fn process_command(commands: Vec<Command>, state: &mut ShellState) -> Result<(), ()> {
    for command in &commands {
        if !command.is_valid() {
            stdout()
                .write_all(format!("{}: command not found\n", &command.program).as_bytes())
                .unwrap();
            return Err(());
        }
    }
    execute_command(&commands, state);
    Ok(())
}

fn main() {
    let mut state = ShellState { jobs: vec![] };
    unsafe {
        signal(Signal::SIGTSTP, SigHandler::SigIgn).unwrap();
    }
    loop {
        match read_and_parse() {
            Some(command) => {
                let pipeline = parse_pipelines(command).unwrap();
                let _ = process_command(pipeline.commands, &mut state);
            }
            None => break,
        }
    }
}
