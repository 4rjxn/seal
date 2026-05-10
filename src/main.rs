mod builtins;
mod error;
mod execution;
mod file_completion;
mod lexer;
mod lua_engine;
mod models;
mod parser;
mod prompt;
mod redirection;
mod repl;
mod traits;
mod types;
mod utils;
mod wait_process;

use std::{process::exit, rc::Rc};

use nix::{
    libc::{getpid, setpgid},
    sys::signal::{SigHandler, Signal, signal},
};

use crate::{
    error::ShellResult,
    execution::spawn_pipeline,
    models::{Command, Pipeline, ShellState},
    parser::parse_tokens,
    repl::Repl,
    types::ShellStateType,
    utils::{ok_to_exit, set_terminal_leader},
};

fn process_command(
    mut commands: Vec<Command>,
    background: bool,
    state: ShellStateType,
) -> ShellResult<()> {
    let _ = commands.iter_mut().try_for_each(|c| {
        if !c.find_binary_from_path() {
            return Err(crate::error::ShellError::CommandNotFound(c.program.clone()));
        }
        Ok(())
    });

    spawn_pipeline(&commands, background, state, false);
    Ok(())
}

fn main() {
    set_signals_for_parent();
    unsafe {
        setpgid(getpid(), getpid());
    }
    set_terminal_leader();
    let state = ShellState::new();
    let mut repl = Repl::new().expect("Failed to initialize REPL");
    loop {
        if let Some(pipeline) = get_pipeline(&mut repl, Rc::clone(&state)) {
            let _ = process_command(pipeline.commands, pipeline.background, Rc::clone(&state));
        }
    }
}

fn set_signals_for_parent() {
    unsafe {
        signal(Signal::SIGTSTP, SigHandler::SigIgn).unwrap();
        signal(Signal::SIGTTOU, SigHandler::SigIgn).unwrap();
        signal(Signal::SIGTTIN, SigHandler::SigIgn).unwrap();
    }
}

fn get_pipeline(repl: &mut Repl, state: ShellStateType) -> Option<Pipeline> {
    match repl.read_and_parse() {
        Some(tokens) => Some(parse_tokens(tokens, state.clone())),
        None => {
            if ok_to_exit(state) {
                exit(0);
            }
            println!("There are things to be done..");
            None
        }
    }
}
