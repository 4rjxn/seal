mod builtins;
mod error;
mod execution;
mod file_completion;
mod lexer;
mod models;
mod parser;
mod prompt;
mod redirection;
mod repl;
mod traits;
mod utils;
mod wait_process;

use nix::sys::signal::{SigHandler, Signal, signal};

use crate::{
    error::ShellResult,
    execution::execute_command,
    models::{Command, ShellState},
    parser::parse_pipelines,
    repl::Repl,
    utils::ok_to_exit,
};
use std::io::{Write, stdout};

fn process_command(
    commands: Vec<Command>,
    background: bool,
    state: &mut ShellState,
) -> ShellResult<()> {
    for command in &commands {
        if !command.is_valid() {
            stdout()
                .write_all(format!("{}: command not found\n", &command.program).as_bytes())
                .unwrap();
            return Err(crate::error::ShellError::CommandNotFound(
                command.program.clone(),
            ));
        }
    }
    execute_command(&commands, background, state);
    Ok(())
}

fn main() {
    let mut state = ShellState {
        jobs: vec![],
        recent_id: 0,
    };
    let mut repl = Repl::new().expect("Failed to initialize REPL");

    unsafe {
        signal(Signal::SIGTSTP, SigHandler::SigIgn).unwrap();
    }
    loop {
        match repl.read_and_parse() {
            Some(tokens) => {
                if let Some(pipeline) = parse_pipelines(tokens) {
                    let _ = process_command(pipeline.commands, pipeline.background, &mut state);
                }
            }
            None => {
                if ok_to_exit(&mut state) {
                    break;
                }
                println!("There are things to be done..");
            }
        }
    }
}
