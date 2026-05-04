use std::{
    ffi::CString,
    os::fd::{AsRawFd, OwnedFd},
    process,
};

use nix::{
    libc::{STDIN_FILENO, STDOUT_FILENO, dup2, getpgrp, setpgid, tcsetpgrp},
    sys::{
        signal::{SigHandler, Signal, signal},
        wait::{WaitStatus, waitpid},
    },
    unistd::{Pid, close, execvp, fork, pipe},
};

use crate::{
    builtins::run_builtin,
    models::{Builtins, Command, CommandKind, ShellState},
    redirection::set_redirection,
};

pub fn spawn_pipeline(commands: &Vec<Command>, background: bool, state: &mut ShellState) {
    let mut pgid: Option<Pid> = None;
    let mut prev_read: Option<OwnedFd> = None;
    let mut children = Vec::new();
    let mut iter = commands.iter().peekable();
    while let Some(command) = iter.next() {
        let has_next = iter.peek().is_some();
        let (read_end, write_end) = make_pipe_if_needed(has_next);
        match command.kind() {
            CommandKind::Builtin(builtin) => {
                run_builtin_inline(command, state, builtin, prev_read, write_end);
                prev_read = read_end;
            }
            CommandKind::External => {
                spawn_external(
                    command,
                    &mut pgid,
                    prev_read,
                    write_end,
                    read_end,
                    &mut children,
                );
                prev_read = None;
            }
        }
    }
    wait_for_children(&children, pgid);
}

fn wait_for_children(children: &[Pid], pgid: Option<Pid>) {
    for &child in children {
        match waitpid(child, None) {
            Ok(status) => handle_wait_status(status),
            Err(e) => eprintln!("waitpid failed: {}", e),
        }
    }
    // give terminal control back to the shell
    if let Some(_) = pgid {
        unsafe { tcsetpgrp(STDIN_FILENO, getpgrp()) };
    }
}
fn handle_wait_status(status: WaitStatus) {
    match status {
        WaitStatus::Exited(_, code) if code != 0 => {
            eprintln!("process exited with code {}", code);
        }
        WaitStatus::Signaled(_, sig, _) => {
            eprintln!("process killed by signal {}", sig);
        }
        _ => {}
    }
}

fn run_builtin_inline(
    command: &Command,
    state: &mut ShellState,
    builtin: Builtins,
    stdin: Option<OwnedFd>,
    stdout: Option<OwnedFd>,
) {
    unsafe { wire_fds(stdin, stdout, None) }
    run_builtin(&command, state, builtin);
}

fn spawn_external(
    command: &Command,
    pgid: &mut Option<Pid>,
    prev_read: Option<OwnedFd>,
    write_end: Option<OwnedFd>,
    read_end: Option<OwnedFd>,
    children: &mut Vec<Pid>,
) {
    match unsafe { fork() } {
        Ok(nix::unistd::ForkResult::Child) => {
            child_setup(*pgid, prev_read, write_end, read_end);
            child_exec(command);
        }
        Ok(nix::unistd::ForkResult::Parent { child }) => {
            parent_cleanup(child, pgid, prev_read, write_end, children);
        }
        Err(e) => eprintln!("Fork failed!! err: {}", e),
    }
}

fn parent_cleanup(
    child: Pid,
    pgid: &mut Option<Pid>,
    prev_read: Option<OwnedFd>,
    write_end: Option<OwnedFd>,
    children: &mut Vec<Pid>,
) {
    if pgid.is_none() {
        *pgid = Some(child);
    }
    if let Some(r) = prev_read {
        close(r).unwrap();
    }
    if let Some(w) = write_end {
        close(w).unwrap();
    }
    children.push(child);
}

fn child_setup(
    pgid: Option<Pid>,
    prev_read: Option<OwnedFd>,
    write_end: Option<OwnedFd>,
    read_end: Option<OwnedFd>,
) {
    unsafe {
        reset_child_signals();
        join_or_create_pgroup(pgid);
        wire_fds(prev_read, write_end, read_end);
    }
}

fn child_exec(command: &Command) {
    if let Err(e) = set_redirection(command) {
        eprintln!("redirection error!! err: {}", e);
        process::exit(1);
    }
    let c = CString::new(command.program.as_bytes()).unwrap();
    let cargs: Vec<CString> = command
        .args
        .iter()
        .map(|a| CString::new(a.as_bytes()).unwrap())
        .collect();
    execvp(&c, &cargs).unwrap();
    process::exit(1);
}

unsafe fn join_or_create_pgroup(pgid: Option<Pid>) {
    unsafe {
        let target = pgid.unwrap_or(Pid::from_raw(0));
        setpgid(Pid::from_raw(0).as_raw(), target.as_raw());
    }
}

fn reset_child_signals() {
    unsafe {
        signal(Signal::SIGINT, SigHandler::SigDfl).unwrap();
        signal(Signal::SIGTERM, SigHandler::SigDfl).unwrap();
        signal(Signal::SIGTSTP, SigHandler::SigDfl).unwrap();
    }
}

unsafe fn wire_fds(stdin: Option<OwnedFd>, stdout: Option<OwnedFd>, read_end: Option<OwnedFd>) {
    if let Some(r) = &stdin {
        unsafe {
            dup2(r.as_raw_fd(), STDIN_FILENO);
            close(r.as_raw_fd()).unwrap();
        }
    }
    if let Some(w) = stdout {
        unsafe {
            dup2(w.as_raw_fd(), STDOUT_FILENO);
            close(w.as_raw_fd()).unwrap();
        }
    }
    if let Some(r) = read_end {
        close(r).unwrap();
    }
}

fn make_pipe_if_needed(has_next: bool) -> (Option<OwnedFd>, Option<OwnedFd>) {
    if !has_next {
        return (None, None);
    }
    match pipe() {
        Ok((read_end, write_end)) => (Some(read_end), Some(write_end)),
        Err(e) => {
            eprintln!("pipe creation failed! err: {}", e);
            return (None, None);
        }
    }
}
