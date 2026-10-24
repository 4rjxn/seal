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

fn child_exec(shell: &mut Shell, cmd: &Command) {
    match cmd {
        Command::Unit(unit_command) => {
            if let Err(e) = apply_redirects(&unit_command.redirects) {
                eprintln!("seal: {e}");
                exit(1)
            }
            if unit_command.words.is_empty() {
                exit(0)
            }
            exec_external(shell, unit_command);
        }

        Command::Subshell(syntax_node) => {
            let code = execute(shell, syntax_node);
            exit(code)
        }
    }
}

fn apply_redirects(redirects: &Vec<Redirect>) -> io::Result<()> {
    todo!()
}

fn wait_pipeline(shell: &mut Shell, pgid: Pid, children: Vec<Pid>, is_background: bool) -> i32 {
    todo!()
}

fn make_pipe_if_needed(has_next: bool) -> Result<(Option<OwnedFd>, Option<OwnedFd>), Error> {
    if !has_next {
        return Ok((None, None));
    }
    pipe2(OFlag::O_CLOEXEC).map(|(r, w)| (Some(r), Some(w)))
}

fn spawn_child(
    shell: &mut Shell,
    cmd: &Command,
    pgid: Option<Pid>,
    prev_read: Option<&OwnedFd>,
    write_end: Option<&OwnedFd>,
) -> nix::Result<Pid> {
    match unsafe { fork()? } {
        nix::unistd::ForkResult::Child => {
            reset_child_signals();

            let target_pgid = pgid.unwrap_or(Pid::from_raw(0));
            let _ = setpgid(Pid::from_raw(0), target_pgid);

            if let Some(r) = prev_read {
                let _ = dup2_stdin(r);
            }
            if let Some(w) = write_end {
                let _ = dup2_stdout(w);
            }

            child_exec(shell, cmd);
            exit(1); //exit if child fails to replace image.
        }
        nix::unistd::ForkResult::Parent { child } => Ok(child),
    }
}

fn exec_external(shell: &mut Shell, cmd: &UnitCommand) {}

fn reset_child_signals() {}
