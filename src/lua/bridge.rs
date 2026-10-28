use std::cell::RefCell;

use mlua::{Function, Lua, Scope, Table};

use crate::{lua::builtins::LuaBuiltin, shell::Shell};

pub fn setup_lua_scope<'lua, 'scope>(
    shell: &'scope RefCell<&mut Shell>,
    scope: &'lua Scope<'lua, 'scope>,
    g_table: Table,
    lua: &'lua Lua,
) -> mlua::Result<()> {
    let register = scope.create_function(|_, (name, help, func): (String, String, Function)| {
        shell.borrow_mut().builtins.register(Box::new(LuaBuiltin {
            name,
            help,
            func,
            lua: lua.clone(),
        }));
        Ok(())
    })?;
    let run = scope.create_function(|_, script: String| {
        let status = shell.borrow_mut().execute_str(&script);
        Ok(status)
    })?;
    g_table.set("register", register)?;
    g_table.set("run", run)?;
    Ok(())
}
