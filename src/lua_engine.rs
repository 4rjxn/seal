use std::{cell::RefCell, rc::Rc};

use mlua::Lua;

use crate::{execution::spawn_pipeline, lexer::tokenize, models::ShellState, parser::parse_tokens};

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
        let run = self
            .lua
            .create_function(move |_, command: String| {
                let tokens = tokenize(command.as_str());
                let command_pipeline = parse_tokens(tokens);
                spawn_pipeline(
                    &command_pipeline.commands,
                    command_pipeline.background,
                    Rc::clone(&state),
                );
                Ok(())
            })
            .unwrap();
        globals.set("run", run).unwrap();
    }
    pub fn run_luastr(&self, script: &str) -> Result<(), mlua::Error> {
        self.lua.load(script).exec()
    }
}
