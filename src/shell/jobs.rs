use nix::{
    sys::wait::{WaitPidFlag, WaitStatus, waitpid},
    unistd::Pid,
};

pub struct JobTable {
    jobs: Vec<Job>,
    next_id: usize,
}

pub struct Job {
    pub id: usize,
    pub pgid: Pid,
    pub status: JobStatus,
    pub command: String,
    pub children: Vec<Pid>,
}

#[derive(PartialEq)]
pub enum JobStatus {
    Running,
    Suspended,
    Done,
}

impl JobTable {
    pub fn new() -> Self {
        Self {
            jobs: Vec::new(),
            next_id: 1,
        }
    }
    pub fn add(&mut self, mut job: Job) {
        job.id = self.next_id();
        self.jobs.push(job);
    }
    pub fn find_by_pgid(&self, pgid: Pid) -> Option<&Job> {
        self.jobs.iter().find(|&s| s.pgid == pgid)
    }
    fn _find_by_pgid_mut(&mut self, pgid: Pid) -> Option<&mut Job> {
        self.jobs.iter_mut().find(|s| s.pgid == pgid)
    }
    pub fn update_done(&mut self) {
        while let Ok(status) = waitpid(Pid::from_raw(-1), Some(WaitPidFlag::WNOHANG)) {
            match status {
                WaitStatus::Exited(pid, _) | WaitStatus::Signaled(pid, _, _) => {
                    for job in &mut self.jobs {
                        if job.children.contains(&pid) {
                            job.children.retain(|&c| c != pid);

                            if job.children.is_empty() {
                                job.status = JobStatus::Done;
                                println!("[{}] + Done\t{}", job.id, job.command);
                            }
                        }
                    }
                }
                _ => break,
            }
            self.remove_done();
        }
    }
    fn remove_done(&mut self) {
        self.jobs.retain(|f| f.status != JobStatus::Done)
    }
    fn next_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}
