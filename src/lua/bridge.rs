use std::cell::RefCell;

use mlua::{Function, Scope, Table};

use crate::{
    lua::builtins::LuaBuiltin,
    shell::{Shell, builtins::ShellContext},
};

pub fn setup_lua_scope<'lua, 'scope>(
    shell: &'scope RefCell<&mut Shell>,
    scope: &'lua Scope<'lua, 'scope>,
    g_table: Table,
) -> mlua::Result<()> {
    let register =
        scope.create_function_mut(|_, (name, help, func): (String, String, Function)| {
            shell
                .borrow_mut()
                .builtins
                .register(Box::new(LuaBuiltin { name, help, func }));
            Ok(())
        })?;
    let run = scope.create_function_mut(|_, script: String| {
        shell.borrow_mut().execute_str(&script);
        Ok(())
    })?;
    g_table.set("register", register)?;
    g_table.set("run", run)?;
    Ok(())
}

