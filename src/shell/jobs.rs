use std::fmt::Display;

use nix::{
    sys::{signal::Signal, wait::WaitStatus},
    unistd::Pid,
};

#[derive(Default)]
pub struct JobTable {
    jobs: Vec<Job>,
}

enum ProcState {
    Running,
    Stopped,
    Exitted(i32),
    Signaled(Signal),
}

struct Process {
    pid: Pid,
    state: ProcState,
}

pub struct Job {
    pub id: usize,
    pub pgid: Pid,
    pub status: JobState,
    pub command: String,
    pub proc: Vec<Process>,
    pub background: bool,
    pub notified: bool,
}

impl Job {
    pub fn state(&self) -> JobState {
        todo!()
    }

    pub fn exit_code(&self) -> i32 {
        todo!()
    }
    pub fn was_interrupted(&self) -> bool {
        todo!()
    }
}

pub enum JobEvent {
    Stopped {
        id: usize,
        command: String,
    },
    Done {
        id: usize,
        command: String,
        exit_code: i32,
    },
}

impl Display for JobEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobEvent::Stopped { id, command } => write!(f, "[{id}]+ Stopped\t{command}"),
            JobEvent::Done { id, command, .. } => write!(f, "[{id}]+ Done\t{command}"),
        }
    }
}

#[derive(PartialEq)]
pub enum JobState {
    Running,
    Stopped,
    Done,
}

impl JobTable {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add(&mut self, pgid: Pid, command: String, childern: &[Pid], background: bool) -> usize {
        todo!()
    }
    pub fn get(&self, id: usize) -> Option<&Job> {
        todo!()
    }
    fn get_mut(&mut self, id: usize) -> Option<&mut Job> {
        todo!()
    }

    pub fn remove(&mut self, id: usize) {}

    pub fn pgids(&self) -> Vec<Pid> {
        todo!()
    }

    pub fn apply_status(&mut self, st: WaitStatus) -> Option<usize> {
        todo!()
    }
    pub fn mark_continued(&mut self, id: usize) {}
    pub fn set_background(&mut self, id: usize, background: bool) {}
    pub fn mark_notified(&mut self, id: usize) {}
    pub fn take_events(&mut self) -> Vec<JobEvent> {
        todo!()
    }
}
