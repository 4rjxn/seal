use nix::unistd::Pid;

use crate::parser::Command;

pub struct ShellState {
    pub jobs: Vec<Job>,
}

pub struct Job {
    pub pgid: Pid,
    pub command: Command,
}
