use crate::models::{Job, JobStatus};

pub fn print_job(job: &Job, recent: usize) {
    let status = match job.status {
        JobStatus::Running => "RUNNING",
        JobStatus::Suspended => "SUSPENDED",
        JobStatus::Done => "DONE",
    };
    println!(
        "[{:>2}]{:<1} {:>6} {:<11}{}",
        job.id,
        if job.id == recent { "+" } else { "" },
        job.pgid,
        status,
        job.command.args[0]
    );
}
