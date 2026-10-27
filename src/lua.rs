use std::{cell::RefCell, fs, path::PathBuf};

use mlua::Lua;
mod bridge;
mod builtins;

use crate::{lua::bridge::setup_lua_scope, shell::Shell};

pub struct LuaEngine {
    lua: Lua,
}

impl LuaEngine {
    pub fn new() -> mlua::Result<Self> {
        let lua = Lua::new();
        let seal = lua.create_table()?;
        lua.globals().set("seal", seal)?;
        Ok(Self { lua: lua })
    }
    pub fn run_luafile(&self, shell: &mut Shell, file: PathBuf) -> mlua::Result<()> {
        let script = fs::read_to_string(file)?;
        self.run_script(shell, &script)
    }

    fn run_script(&self, shell: &mut Shell, script: &str) -> mlua::Result<()> {
        let shell = RefCell::new(shell);
        let shell_ref = &shell;
        self.lua.scope(|scope| {
            setup_lua_scope(shell_ref, scope, self.lua.globals().get("seal")?, &self.lua)?;
            self.lua.load(script).exec()?;
            Ok(())
        })
    }
}
