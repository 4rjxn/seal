use mlua::Function;

use crate::shell::builtins::BuiltIn;

pub struct LuaBuiltin {
    pub name: String,
    pub help: String,
    pub func: Function,
}

impl BuiltIn for LuaBuiltin {
    fn name(&self) -> &str {
        &self.name
    }

    fn help(&self) -> &str {
        &self.help
    }

    fn run(&self, _ctx: &mut crate::shell::builtins::ShellContext, args: &[String]) -> i32 {
        match self.func.call::<Option<i32>>(args.to_vec()) {
            Ok(s) => s.unwrap_or(0),
            Err(e) => {
                eprintln!("{e}");
                1
            }
        }
    }
}
