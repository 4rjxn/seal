use std::{collections::HashMap, io::Write};

use nix::unistd::Pid;

use crate::shell::{
    Shell,
    builtins::{cd::Cd, exit::Exit},
    env::Env,
    jobs::JobTable,
    opts::ShellOpts,
};

mod cd;
mod exit;

//the helper struct which holds mutable refrence to shell different fields.
pub struct ShellContext<'a> {
    pub env: &'a mut Env,
    pub jobs: &'a mut JobTable,
    pub last_exit: &'a mut i32,
    pub shell_pgid: Pid,
    pub shell_opts: &'a mut ShellOpts,
}

//the builtin trait every builtin should
//implement.
pub trait BuiltIn {
    fn name(&self) -> &str;
    fn help(&self) -> &str;
    fn run(&self, ctx: &mut ShellContext, args: &[String]) -> i32;
}

#[derive(Default)]
pub struct BuiltinRegistry {
    map: HashMap<String, Box<dyn BuiltIn>>,
}

impl BuiltinRegistry {
    pub fn with_default() -> Self {
        let mut r = Self::default();
        r.register(Box::new(Exit));
        r.register(Box::new(Cd));
        r
    }
    pub fn register(&mut self, b: Box<dyn BuiltIn>) {
        self.map.insert(b.name().to_string(), b);
    }
    pub fn get(&self, name: &str) -> Option<&dyn BuiltIn> {
        self.map.get(name).map(|b| b.as_ref())
    }
}

pub fn run_builtin(shell: &mut Shell, args: &[String]) -> Option<i32> {
    let Shell {
        env,
        jobs,
        last_exit,
        shell_pgid,
        builtins,
        shell_opts,
    } = shell;
    let b = builtins.get(args.first()?)?;
    let mut ctx = ShellContext {
        env,
        jobs,
        last_exit,
        shell_pgid: *shell_pgid,
        shell_opts,
    };
    let exit_code = b.run(&mut ctx, args);
    let _ = std::io::stdout().flush();
    Some(exit_code)
}
