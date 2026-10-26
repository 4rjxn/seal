use crate::shell::{builtins::BuiltIn, job_control::wait_foreground};

pub struct Fg;

impl BuiltIn for Fg {
    fn name(&self) -> &str {
        "fg"
    }

    fn help(&self) -> &str {
        "Usage: fg [job id]\n\
        \n\
        fg is a builtin command\n\
        it foregrounds the latest job or job of a given id.\n\
        currently it can only foreground the latest job.\n\
        (i havent implemented fg based on id yet :)"
    }

    fn run(&self, ctx: &mut super::ShellContext, _args: &[String]) -> i32 {
        // this is not a complete implementation, this is just built to get a quick 'fg'
        // command since fg is  something i frequently use. for now fg only foregrounds the
        // most recent job form the jobtable
        //give_terminal_to(ctx.shell_opts.is_interactive);
        if !ctx.shell_opts.is_interactive {
            eprintln!("seal: fg: no jobs to foreground");
            return 1;
        }
        let Some(job) = ctx.jobs.current_job() else {
            eprintln!("seal: fg: no jobs to foreground");
            return 1;
        };
        let _ = wait_foreground(ctx.jobs, job.id, true, true, ctx.shell_pgid);
        0
    }
}
