use std::process::exit;

use crate::shell::builtins::BuiltIn;

pub struct Exit;

impl BuiltIn for Exit {
    fn name(&self) -> &str {
        "exit"
    }

    fn help(&self) -> &str {
        "exit the shell"
    }

    fn run(&self, ctx: &mut super::ShellContext, _args: &[String]) -> i32 {
        exit(*ctx.last_exit);
    }
}
