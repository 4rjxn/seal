use crate::shell::{Shell, builtins::BuiltIn};

pub struct Job;

impl BuiltIn for Job {
    fn name(&self) -> &str {
        "job"
    }

    fn help(&self) -> &str {
        "Usage: job\n\
        \n\
        prints jobs present in jobtable."
    }

    fn run(&self, ctx: &mut Shell, _args: &[String]) -> i32 {
        if !ctx.shell_opts.is_interactive {
            return 0;
        }
        for job in ctx.jobs.iter() {
            println!("{job}");
        }
        0
    }
}
