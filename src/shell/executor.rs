use std::{
    io,
    os::fd::{AsRawFd, FromRawFd, OwnedFd},
    process::exit,
};

use nix::{
    Error,
    fcntl::OFlag,
    libc::STDIN_FILENO,
    sys::signal::{Signal::SIGTERM, kill},
    unistd::{Pid, dup2, dup2_stdin, dup2_stdout, fork, pipe, pipe2, setpgid},
};

use crate::{
    parser::node::{Command, Redirect, SyntaxNode, UnitCommand},
    shell::Shell,
};

fn execute(shell: &mut Shell, syntax: &SyntaxNode) -> i32 {
    todo!()
}

fn execute_and_or(shell: &mut Shell, left: SyntaxNode, right: SyntaxNode, is_and: bool) -> i32 {
    todo!()
}

fn execute_sequence(shell: &mut Shell, nodes: Vec<SyntaxNode>) -> i32 {
    todo!()
}

fn execute_pipeline(shell: &mut Shell, cmds: Vec<Command>, is_background: bool) -> i32 {
    let mut prev_read: Option<OwnedFd> = None;
    let mut pgid: Option<Pid> = None;
    let mut children = Vec::new();
    let mut iter = cmds.iter().peekable();
    while let Some(command) = iter.next() {
        let has_next = iter.peek().is_some();
        let (read_end, write_end) = match make_pipe_if_needed(has_next) {
            Ok(val) => val,
            Err(e) => {
                eprintln!("seal: pipe creation failed {e}");
                drop(prev_read);
                if let Some(job_pgid) = pgid {
                    let _ = kill(job_pgid, SIGTERM);
                    let _ = wait_pipeline(shell, job_pgid, children, is_background);
                }
                return 1;
            }
        };
        let child = match spawn_child(
            shell,
            &command,
            pgid,
            prev_read.as_ref(),
            write_end.as_ref(),
        ) {
            Ok(child) => child,
            Err(e) => {
                eprintln!("seal: fork failed: {e}");
                drop(write_end);
                drop(prev_read);
                if let Some(job_pgid) = pgid {
                    let _ = kill(job_pgid, SIGTERM);
                    let _ = wait_pipeline(shell, job_pgid, children, is_background);
                }
                return 1;
            }
        };
        if pgid.is_none() {
            pgid = Some(child);
        }
        if let Some(job_pgid) = pgid {
            let _ = setpgid(child, job_pgid);
        }
        children.push(child);
        drop(write_end);
        drop(prev_read);

        prev_read = read_end;
    }
    if let Some(job_pgid) = pgid {
        wait_pipeline(shell, job_pgid, children, is_background)
    } else {
        return 0;
    }
}

fn child_exec(shell: &mut Shell, cmd: Command) {}

fn apply_redirects(redirects: Vec<Redirect>) -> io::Result<()> {
    todo!()
}

fn wait_pipeline(shell: &mut Shell, pgid: Pid, children: Vec<Pid>, is_background: bool) -> i32 {
    todo!()
}
