use std::{
    ffi::CString,
    os::fd::{AsRawFd, OwnedFd},
    process,
};

use nix::{
    libc::{STDIN_FILENO, STDOUT_FILENO, dup2, setpgid},
    sys::signal::{SigHandler, Signal, signal},
    unistd::{Pid, execvp, fork, pipe},
};

use crate::{
    builtins::run_builtin,
    models::{Builtins, Command, CommandKind, Job, JobStatus, ShellState},
    redirection::set_redirection,
    utils::is_builtin,
    wait_process::wait_for_process,
};

pub fn spawn_pipeline(commands: &Vec<Command>, _background: bool, state: &mut ShellState) {
    let mut pgid: Option<Pid> = None;
    let mut prev_read: Option<OwnedFd> = None;
    let mut children = Vec::new();
    let mut iter = commands.iter().peekable();
    while let Some(command) = iter.next() {
        let has_next = iter.peek().is_some();
        let (read_end, write_end) = make_pipe_if_needed(has_next);
        match command.kind() {
            CommandKind::Builtin(builtin) => {
                run_builtin_inline(
                    command,
                    state,
                    builtin,
                    prev_read.as_ref(),
                    write_end.as_ref(),
                );
                prev_read = read_end;
            }
            CommandKind::External => {
                spawn_external(
                    command,
                    &mut pgid,
                    prev_read.as_ref(),
                    write_end.as_ref(),
                    read_end.as_ref(),
                    &mut children,
                );
                prev_read = read_end;
            }
        }
    }

    if let Some(job) = generate_job(pgid, state, commands.last().unwrap(), &mut children) {
        wait_for_process(job, state);
    }
}

fn generate_job(
    pgid: Option<Pid>,
    state: &mut ShellState,
    command: &Command,
    children: &mut Vec<Pid>,
) -> Option<Job> {
    if is_builtin(&command.program).is_some() {
        return None;
    }
    let job = Job {
        id: state.jobs.len() + 1,
        pgid: Pid::from_raw(pgid.unwrap().into()),
        status: JobStatus::Running,
        command: command.clone(),
        childrens: children.to_owned(),
    };
    Some(job)
}

fn run_builtin_inline(
    command: &Command,
    state: &mut ShellState,
    builtin: Builtins,
    stdin: Option<&OwnedFd>,
    stdout: Option<&OwnedFd>,
) {
    unsafe { wire_fds(stdin, stdout, None) }
    let _ = run_builtin(&command, state, builtin);
}

fn spawn_external(
    command: &Command,
    pgid: &mut Option<Pid>,
    prev_read: Option<&OwnedFd>,
    write_end: Option<&OwnedFd>,
    read_end: Option<&OwnedFd>,
    children: &mut Vec<Pid>,
) {
    match unsafe { fork() } {
        Ok(nix::unistd::ForkResult::Child) => {
            child_setup(*pgid, prev_read, write_end, read_end);
            child_exec(command);
        }
        Ok(nix::unistd::ForkResult::Parent { child }) => {
            parent_cleanup(child, pgid, children);
        }
        Err(e) => eprintln!("Fork failed!! err: {}", e),
    }
}

fn parent_cleanup(child: Pid, pgid: &mut Option<Pid>, children: &mut Vec<Pid>) {
    if pgid.is_none() {
        *pgid = Some(child);
    }
    children.push(child);
}

fn child_setup(
    pgid: Option<Pid>,
    prev_read: Option<&OwnedFd>,
    write_end: Option<&OwnedFd>,
    read_end: Option<&OwnedFd>,
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
    match execvp(&c, &cargs) {
        Ok(_) => {}
        Err(_) => {
            println!("{}: command not found", command.program)
        }
    }
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

unsafe fn wire_fds(stdin: Option<&OwnedFd>, stdout: Option<&OwnedFd>, read_end: Option<&OwnedFd>) {
    if let Some(r) = stdin {
        unsafe {
            dup2(r.as_raw_fd(), STDIN_FILENO);
        }
    }
    if let Some(w) = stdout {
        unsafe {
            dup2(w.as_raw_fd(), STDOUT_FILENO);
        }
    }
    if let Some(_r) = read_end {
        // No need to close here, OwnedFd will handle it or it will be closed on exec
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
