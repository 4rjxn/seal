use std::fmt::Display;

use nix::{
    sys::{
        signal::Signal::{self, SIGINT, SIGTSTP},
        wait::WaitStatus,
    },
    unistd::Pid,
};

#[derive(Default)]
pub struct JobTable {
    jobs: Vec<Job>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProcState {
    Running,
    Stopped,
    Exitted(i32),
    Signaled(Signal),
}

pub struct Process {
    pid: Pid,
    state: ProcState,
}

pub struct Job {
    pub id: usize,
    pub pgid: Pid,
    pub command: String,
    pub proc: Vec<Process>,
    pub background: bool,
    pub notified: bool,
}

impl Job {
    pub fn state(&self) -> JobState {
        if self.proc.iter().any(|p| p.state == ProcState::Running) {
            JobState::Running
        } else if self.proc.iter().any(|p| p.state == ProcState::Stopped) {
            JobState::Stopped
        } else {
            JobState::Done
        }
    }

    pub fn exit_code(&self) -> i32 {
        match self.proc.last().map(|p| p.state) {
            Some(ProcState::Exitted(c)) => c,
            Some(ProcState::Signaled(sig)) => 128 + sig as i32,
            Some(ProcState::Stopped) => 128 + SIGTSTP as i32,
            _ => 0,
        }
    }
    pub fn was_interrupted(&self) -> bool {
        self.proc
            .iter()
            .any(|p| p.state == ProcState::Signaled(SIGINT))
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
        let id = self.jobs.iter().map(|j| j.id).max().unwrap_or(0) + 1;
        let procs = childern
            .iter()
            .map(|&pid| Process {
                pid,
                state: ProcState::Running,
            })
            .collect();
        self.jobs.push(Job {
            id,
            pgid,
            command,
            proc: procs,
            background,
            notified: false,
        });
        id
    }
    pub fn get(&self, id: usize) -> Option<&Job> {
        self.jobs.iter().find(|j| j.id == id)
    }
    fn get_mut(&mut self, id: usize) -> Option<&mut Job> {
        self.jobs.iter_mut().find(|j| j.id == id)
    }

    pub fn remove(&mut self, id: usize) {
        self.jobs.retain(|j| j.id != id);
    }

    pub fn pgids(&self) -> Vec<Pid> {
        self.jobs
            .iter()
            .filter(|j| j.state() != JobState::Done)
            .map(|j| j.pgid)
            .collect()
    }

    pub fn apply_status(&mut self, st: WaitStatus) -> Option<usize> {
        let (pid, new) = match st {
            WaitStatus::Exited(p, c) => (p, ProcState::Exitted(c)),
            WaitStatus::Signaled(p, s, _) => (p, ProcState::Signaled(s)),
            WaitStatus::Stopped(p, _) => (p, ProcState::Stopped),
            WaitStatus::Continued(p) => (p, ProcState::Running),
            _ => return None,
        };
        for job in &mut self.jobs {
            if let Some(proc) = job.proc.iter_mut().find(|p| p.pid == pid) {
                proc.state = new;
                if new == ProcState::Running {
                    job.notified = false
                }
                return Some(job.id);
            }
        }
        None
    }
    pub fn mark_continued(&mut self, id: usize) {
        if let Some(job) = self.get_mut(id) {
            for p in &mut job.proc {
                if p.state == ProcState::Stopped {
                    p.state = ProcState::Running
                }
            }
            job.notified = false;
        }
    }

    pub fn set_background(&mut self, id: usize, background: bool) {
        if let Some(job) = self.get_mut(id) {
            job.background = background;
        }
    }
    pub fn mark_notified(&mut self, id: usize) {
        if let Some(job) = self.get_mut(id) {
            job.notified = true
        }
    }
    pub fn take_events(&mut self) -> Vec<JobEvent> {
        let mut events = Vec::new();
        for job in &mut self.jobs {
            match job.state() {
                JobState::Stopped if !job.notified => {
                    job.notified = true;
                    events.push(JobEvent::Stopped {
                        id: job.id,
                        command: job.command.clone(),
                    })
                }
                JobState::Done if job.background => events.push(JobEvent::Done {
                    id: job.id,
                    command: job.command.clone(),
                    exit_code: job.exit_code(),
                }),
                _ => {}
            }
        }
        self.jobs.retain(|j| j.state() != JobState::Done);
        events
    }
}
