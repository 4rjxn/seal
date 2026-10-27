use std::{collections::HashMap, io::Write, rc::Rc};

use nix::unistd::Pid;

use crate::shell::{
    Shell,
    builtins::{cd::Cd, exit::Exit, fg::Fg, job::Job, summon::Summon},
    env::Env,
    jobs::JobTable,
    opts::ShellOpts,
};

mod cd;
mod exit;
mod fg;
mod job;
mod summon;

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
    fn run(&self, ctx: &mut Shell, args: &[String]) -> i32;
}

#[derive(Default)]
pub struct BuiltinRegistry {
    map: HashMap<String, Rc<dyn BuiltIn>>,
}

impl BuiltinRegistry {
    pub fn with_default() -> Self {
        let mut r = Self::default();
        r.register(Box::new(Exit));
        r.register(Box::new(Cd));
        r.register(Box::new(Job));
        r.register(Box::new(Summon));
        r.register(Box::new(Fg));
        r
    }
    pub fn register(&mut self, b: Box<dyn BuiltIn>) {
        self.map.insert(b.name().to_string(), Rc::from(b));
    }
    pub fn get(&self, name: &str) -> Option<Rc<dyn BuiltIn>> {
        self.map.get(name).cloned()
    }
}

pub fn run_builtin(shell: &mut Shell, args: &[String]) -> Option<i32> {
    let b = shell.builtins.get(args.first()?)?.clone();
    if args.get(1).map(String::as_str) == Some("--help") {
        println!("{}", b.help());
        return Some(0);
    }
    let exit_code = b.run(shell, args);
    let _ = std::io::stdout().flush();
    Some(exit_code)
}
