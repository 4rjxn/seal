use std::process::exit;

use crate::shell::builtins::BuiltIn;

pub struct Exit;

impl BuiltIn for Exit {
    fn name(&self) -> &str {
        "exit"
    }

    fn help(&self) -> &str {
        "Usage: exit [code]\n\
        \n\
         exit the shell with given code.\n\
         if there is no code provided the exit code of last command will be used"
    }

    fn run(&self, ctx: &mut super::ShellContext, _args: &[String]) -> i32 {
        exit(*ctx.last_exit);
    }
}
