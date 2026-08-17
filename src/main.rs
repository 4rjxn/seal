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
    execution::spawn_pipeline,
    models::{Pipeline, ShellState},
    parser::parse_tokens,
    repl::Repl,
    types::ShellStateType,
    utils::{ok_to_exit, set_terminal_leader},
};

fn main() {
    set_signals_for_parent();
    unsafe {
        setpgid(getpid(), getpid());
    }
    set_terminal_leader();
    let state = ShellState::new();
    let mut repl = Repl::new().expect("Failed to initialize REPL");
    loop {
        if let Some(mut pipeline) = get_pipeline(&mut repl, Rc::clone(&state)) {
            pipeline.process_pipeline();
            spawn_pipeline(
                &pipeline.commands,
                pipeline.background,
                Rc::clone(&state),
                false,
            );
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
