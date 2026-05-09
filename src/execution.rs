use std::{
    ffi::CString,
    fs::File,
    io::Read,
    os::fd::{AsRawFd, FromRawFd, IntoRawFd, OwnedFd},
    process,
    rc::Rc,
};

use nix::{
    libc::{STDIN_FILENO, STDOUT_FILENO, dup2, setpgid},
    sys::signal::{SigHandler, Signal, signal},
    unistd::{Pid, execvp, fork, pipe},
};

use crate::{
    builtins::run_builtin,
    models::{Builtins, Command, CommandKind, Job, JobStatus},
    redirection::set_redirection,
    types::ShellStateType,
    utils::{generate_cmd_cargs, give_terminal_to_job, is_builtin},
    wait_process::wait_for_process,
};

pub fn spawn_pipeline(
    commands: &Vec<Command>,
    _background: bool,
    state: ShellStateType,
    capture: bool,
) -> Option<Vec<u8>> {
    let mut pgid: Option<Pid> = None;
    let mut prev_read: Option<OwnedFd> = None;
    let mut children = Vec::new();
    let mut iter = commands.iter().peekable();
    let (capture_read, capture_write) = make_capture_pip(capture);
    while let Some(command) = iter.next() {
        let has_next = iter.peek().is_some();
        let (read_end, write_end) = make_pipe_if_needed(has_next);
        let effective_write = if !has_next && capture {
            capture_write.as_ref()
        } else {
            write_end.as_ref()
        };
        match command.kind() {
            CommandKind::Builtin(builtin) => {
                run_builtin_inline(
                    command,
                    Rc::clone(&state),
                    builtin,
                    prev_read.as_ref(),
                    effective_write,
                );
                prev_read = read_end;
            }
            CommandKind::External => {
                spawn_external(
                    command,
                    &mut pgid,
                    prev_read.as_ref(),
                    effective_write,
                    read_end.as_ref(),
                    &mut children,
                );
                prev_read = read_end;
            }
        }
    }
    if capture {
        // Drop the write end so reading doesn't block
        drop(capture_write);

        let mut output = Vec::new();
        let mut file = unsafe { File::from_raw_fd(capture_read.unwrap().into_raw_fd()) };
        file.read_to_end(&mut output).ok();

        // Wait for children as usual...
        if let Some(job) = generate_job(
            pgid,
            Rc::clone(&state),
            commands.last().unwrap(),
            &mut children,
        ) {
            give_terminal_to_job(job.pgid);
            wait_for_process(job, state);
        }

        return Some(output);
    }

    if let Some(job) = generate_job(
        pgid,
        Rc::clone(&state),
        commands.last().unwrap(),
        &mut children,
    ) {
        give_terminal_to_job(job.pgid);
        wait_for_process(job, state);
    }
    None
}

fn generate_job(
    pgid: Option<Pid>,
    state: ShellStateType,
    command: &Command,
    children: &mut Vec<Pid>,
) -> Option<Job> {
    let state = state.borrow_mut();
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
    state: ShellStateType,
    builtin: Builtins,
    read_fd: Option<&OwnedFd>,
    write_fd: Option<&OwnedFd>,
) {
    // Save original stdout/stdin
    let saved_stdout = write_fd.map(|_| {
        let fd = unsafe { OwnedFd::from_raw_fd(nix::libc::dup(1)) };
        fd
    });
    let saved_stdin = read_fd.map(|_| {
        let fd = unsafe { OwnedFd::from_raw_fd(nix::libc::dup(0)) };
        fd
    });

    // Redirect stdin/stdout to the pipe fds
    unsafe {
        if let Some(rfd) = read_fd {
            nix::libc::dup2(rfd.as_raw_fd(), 0);
        }
        if let Some(wfd) = write_fd {
            nix::libc::dup2(wfd.as_raw_fd(), 1);
        }
    }

    // Run the builtin — it writes to fd 1 as usual
    let _ = run_builtin(&command, state, builtin);

    // Restore original stdout/stdin
    unsafe {
        if let Some(saved) = saved_stdout {
            nix::libc::dup2(saved.as_raw_fd(), 1);
        }
        if let Some(saved) = saved_stdin {
            nix::libc::dup2(saved.as_raw_fd(), 0);
        }
    }
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
    let (cmd, cargs) = generate_cmd_cargs(&command.program, &command.args);
    exec_command(cmd, cargs);
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

fn make_capture_pip(capture: bool) -> (Option<OwnedFd>, Option<OwnedFd>) {
    if capture {
        let (r, w) = pipe().expect("failed to create capture pipe");
        (Some(r), Some(w))
    } else {
        (None, None)
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

pub fn exec_command(cmd: CString, cargs: Vec<CString>) {
    match execvp(&cmd, &cargs) {
        Ok(_) => {}
        Err(_) => {
            println!("{}: command not found", cmd.to_string_lossy())
        }
    }
}
