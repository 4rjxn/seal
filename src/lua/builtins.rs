use std::cell::RefCell;

use mlua::{Function, Lua};

use crate::{
    lua::bridge::setup_lua_scope,
    shell::{Shell, builtins::BuiltIn},
};

pub struct LuaBuiltin {
    pub name: String,
    pub help: String,
    pub func: Function,
    pub lua: Lua,
}

impl BuiltIn for LuaBuiltin {
    fn name(&self) -> &str {
        &self.name
    }

    fn help(&self) -> &str {
        &self.help
    }

    fn run(&self, ctx: &mut Shell, args: &[String]) -> i32 {
        let shell_ref = RefCell::new(ctx);
        let result = self.lua.scope(|scope| {
            setup_lua_scope(
                &shell_ref,
                scope,
                self.lua.globals().get("seal")?,
                &self.lua,
            )?;
            self.func.call::<Option<i32>>(&args[1..])?;
            Ok(())
        });
        match result {
            Ok(_) => 0,
            Err(_) => 1,
        }
    }
}
