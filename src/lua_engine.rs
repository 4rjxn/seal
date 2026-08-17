use std::{
    cell::RefCell,
    io::{self, Write},
    rc::Rc,
};

use mlua::Lua;

use crate::{
    execution::spawn_pipeline, lexer::tokenize, models::ShellState, parser::parse_tokens,
    types::ShellStateType,
};

pub struct LuaEngine {
    pub lua: Lua,
    state: Rc<RefCell<ShellState>>,
}

impl LuaEngine {
    pub fn new(state: Rc<RefCell<ShellState>>) -> Self {
        let mut engine = LuaEngine {
            lua: Lua::new(),
            state: Rc::clone(&state),
        };
        engine.register_apis();
        engine
    }

    fn register_apis(&mut self) {
        let globals = self.lua.globals();
        let state = Rc::clone(&self.state);
        let run_state = Rc::clone(&state);
        let run = self
            .lua
            .create_function(move |_, command: String| {
                run_pipeline_from_str(&command, Rc::clone(&run_state), false);
                Ok(())
            })
            .unwrap();
        globals.set("run", run).unwrap();
        let capture_state = Rc::clone(&state);
        let capture = self
            .lua
            .create_function(move |_, command: String| {
                match run_pipeline_from_str(&command, capture_state.clone(), true) {
                    Some(capture) => Ok(capture),
                    None => Ok("".to_string()),
                }
            })
            .unwrap();
        globals.set("capture", capture).unwrap();
        let read = self
            .lua
            .create_function(|_, prompt: String| {
                print!("{}", prompt);
                io::stdout().flush().unwrap();

                let mut buffer = String::new();
                io::stdin().read_line(&mut buffer).unwrap();

                Ok(buffer.trim().to_string())
            })
            .unwrap();
        globals.set("read", read).unwrap();
    }
    pub fn run_luastr(&self, script: &str) -> Result<(), mlua::Error> {
        self.lua.load(script).exec()
    }
}

fn run_pipeline_from_str(str: &str, state: ShellStateType, capture: bool) -> Option<String> {
    let tokens = tokenize(str);
    let mut command_pipeline = parse_tokens(tokens, state.clone());
    command_pipeline.process_pipeline();
    let output_data = spawn_pipeline(
        &command_pipeline.commands,
        command_pipeline.background,
        state,
        capture,
    );
    match output_data {
        Some(bytes) => Some(String::from_utf8_lossy(&bytes).into_owned()),
        None => None,
    }
}
