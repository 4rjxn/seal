use std::{
    ffi::CString,
    io::{Write, stderr, stdout},
    os::{
        fd::{BorrowedFd, FromRawFd, OwnedFd, RawFd},
        unix::ffi::OsStrExt,
    },
    path::PathBuf,
    process::exit,
};

use is_executable::IsExecutable;
use nix::{
    Error,
    errno::Errno::EACCES,
    fcntl::{FcntlArg, OFlag, open},
    libc::STDOUT_FILENO,
    sys::{
        signal::{
            SigHandler,
            Signal::{self},
            signal,
        },
        stat::Mode,
    },
    unistd::{Pid, dup2_raw, dup2_stderr, dup2_stdin, dup2_stdout, execve, fork, pipe2, setpgid},
};

use crate::{
    parser::node::{Command, Redirect, RedirectKind, SyntaxNode, UnitCommand},
    shell::{
        Shell,
        builtins::run_builtin,
        job_control::{abort_launch, wait_foreground},
    },
};

pub fn execute(shell: &mut Shell, syntax: &SyntaxNode) -> i32 {
    match syntax {
        SyntaxNode::And { left, right } => execute_and_or(shell, left, right, true),
        SyntaxNode::Or { left, right } => execute_and_or(shell, left, right, false),
        SyntaxNode::Background(syntax_node) => execute_background(shell, syntax_node),
        SyntaxNode::Pipeline(commands) => execute_pipeline(shell, commands, false),
        SyntaxNode::Sequence(nodes) => execute_sequence(shell, nodes),
    }
}

fn execute_and_or(shell: &mut Shell, left: &SyntaxNode, right: &SyntaxNode, is_and: bool) -> i32 {
    let mut last_exit_code;
    if is_and {
        last_exit_code = execute(shell, left);
        if last_exit_code == 0 {
            last_exit_code = execute(shell, right);
        }
    } else {
        last_exit_code = execute(shell, left);
        if last_exit_code != 0 {
            last_exit_code = execute(shell, right);
        }
    }
    last_exit_code
}

fn execute_sequence(shell: &mut Shell, nodes: &[SyntaxNode]) -> i32 {
    let mut last_exit_code = 0;
    for node in nodes {
        last_exit_code = execute(shell, node);
    }
    last_exit_code
}

fn execute_background(shell: &mut Shell, syntax: &SyntaxNode) -> i32 {
    match syntax {
        SyntaxNode::Pipeline(pipeline) => execute_pipeline(shell, pipeline, true),
        complex => match unsafe { fork() } {
            Ok(f) => match f {
                nix::unistd::ForkResult::Child => {
                    let _ = setpgid(Pid::from_raw(0), Pid::from_raw(0));
                    reset_child_signals();

                    shell.shell_opts.monitor_jobs = false;

                    let exit_code = execute(shell, complex);
                    exit(exit_code);
                }
                nix::unistd::ForkResult::Parent { child } => {
                    let _ = setpgid(child, child);
                    let id = shell.jobs.add(child, format!("{child}"), &[child], true);
                    println!("[{id}] {child}",);
                    0 //return successes
                }
            },
            Err(e) => {
                eprintln!("seal: fork failid {e}");
                1 //return faliure
            }
        },
    }
}

fn execute_pipeline(shell: &mut Shell, cmds: &[Command], is_background: bool) -> i32 {
    if cmds.len() == 1 && !is_background {
        let cmd = &cmds[0];
        match cmd {
            Command::Unit(unit_command) => {
                if is_builtin(shell, unit_command) {
                    return run_builtin_with_redirects(shell, unit_command);
                }
            }
            Command::Subshell(_) => {}
        }
    }

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
                abort_launch(pgid, &children);
                return 1;
            }
        };
        let child = match spawn_child(
            shell,
            command,
            pgid,
            prev_read.as_ref(),
            write_end.as_ref(),
            is_background,
        ) {
            Ok(child) => child,
            Err(e) => {
                eprintln!("seal: fork failed: {e}");
                drop(write_end);
                drop(prev_read);
                abort_launch(pgid, &children);
                return 1;
            }
        };
        let leader_pgid = pgid.get_or_insert(child);
        if shell.shell_opts.monitor_jobs {
            let _ = setpgid(child, *leader_pgid);
        }
        children.push(child);
        drop(write_end);
        drop(prev_read);

        prev_read = read_end;
    }
    if let Some(job_pgid) = pgid {
        wait_pipeline(
            shell,
            job_pgid.to_string(),
            job_pgid,
            children,
            is_background,
        )
    } else {
        0 //return successes
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

            if let Some(exit_code) = run_builtin(shell, &unit_command.words) {
                let _ = stdout().flush();
                let _ = stderr().flush();
                exit(exit_code)
            }

            exec_external(shell, unit_command);
        }

        Command::Subshell(syntax_node) => {
            let code = execute(shell, syntax_node);
            exit(code)
        }
    }
}

fn apply_redirects(redirects: &Vec<Redirect>) -> Result<(), String> {
    for redir in redirects {
        match &redir.target {
            crate::parser::node::RedirectTarget::File(file) => {
                let (oflag, default_fd) = match redir.kind {
                    RedirectKind::Input => (
                        OFlag::O_RDONLY,
                        nix::libc::STDIN_FILENO, // 0
                    ),
                    RedirectKind::Output => (
                        OFlag::O_WRONLY | OFlag::O_CREAT | OFlag::O_TRUNC,
                        nix::libc::STDOUT_FILENO, // 1
                    ),
                    RedirectKind::Append => (
                        OFlag::O_WRONLY | OFlag::O_CREAT | OFlag::O_APPEND,
                        nix::libc::STDOUT_FILENO, // 1
                    ),
                    RedirectKind::All => (
                        OFlag::O_WRONLY | OFlag::O_CREAT | OFlag::O_TRUNC,
                        nix::libc::STDOUT_FILENO, // 1 (and 2)
                    ),
                    RedirectKind::AllAppend => (
                        OFlag::O_WRONLY | OFlag::O_CREAT | OFlag::O_APPEND,
                        nix::libc::STDOUT_FILENO, // 1 (and 2)
                    ),
                };

                let mode = Mode::S_IRUSR | Mode::S_IWUSR | Mode::S_IRGRP | Mode::S_IROTH;
                let file_fd =
                    open(file.as_str(), oflag, mode).map_err(|e| format!("{file}: {e}"))?;
                let target_fd = redir.fd.unwrap_or(default_fd);

                //unsafe dup2
                unsafe {
                    // `dup2_raw` returns an `OwnedFd` for `target_fd`. Dropping it
                    // would close the descriptor we just installed, so leak it:
                    // the fd is inherited by the exec'd program.
                    dup2_raw(&file_fd, target_fd)
                        .map(std::mem::forget)
                        .map_err(|e| format!("{file}: {e}"))?;
                }

                if matches!(redir.kind, RedirectKind::All | RedirectKind::AllAppend) {
                    dup2_stderr(&file_fd).map_err(|e| format!("{file}: {e}"))?;
                }
            }
            crate::parser::node::RedirectTarget::Fd(fd) => {
                let src_fd = redir.fd.unwrap_or(STDOUT_FILENO);
                unsafe {
                    dup2_raw(BorrowedFd::borrow_raw(*fd), src_fd)
                        .map(std::mem::forget)
                        .map_err(|e| format!("{e}"))?;
                }
            }
        }
    }
    Ok(())
}

fn wait_pipeline(
    shell: &mut Shell,
    command: String,
    pgid: Pid,
    children: Vec<Pid>,
    is_background: bool,
) -> i32 {
    let id = shell.jobs.add(pgid, command, &children, is_background);
    if is_background {
        println!("[{id}] {pgid}");
        return 0;
    }
    let Some(outcome) = wait_foreground(
        &mut shell.jobs,
        id,
        false,
        shell.shell_opts.monitor_jobs,
        shell.shell_pgid,
    ) else {
        return 1;
    };
    if outcome.interrupted {
        println!();
    }

    if let Some(ev) = outcome.event {
        println!("\n{ev}");
    };
    outcome.exit_code
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
    is_pipeline_bg: bool,
) -> nix::Result<Pid> {
    match unsafe { fork()? } {
        nix::unistd::ForkResult::Child => {
            if shell.shell_opts.monitor_jobs {
                let target_pgid = pgid.unwrap_or(Pid::from_raw(0));
                let _ = setpgid(Pid::from_raw(0), target_pgid);
            }
            if is_pipeline_bg {
                shell.shell_opts.monitor_jobs = false;
            }

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

fn exec_external(shell: &mut Shell, cmd: &UnitCommand) -> ! {
    let program = &cmd.words[0];
    let binary_path = match resolve_binary_path(shell, program) {
        Some(path) => path,
        None => {
            eprintln!("seal: {program}: command not found");
            exit(127)
        }
    };

    let c_path = match CString::new(binary_path.as_os_str().as_bytes()) {
        Ok(path) => path,
        Err(_) => {
            eprintln!("path contains invalid null byte.");
            exit(127);
        }
    };

    let c_args: Vec<CString> = cmd
        .words
        .iter()
        .map(|s| CString::new(s.as_bytes()).unwrap_or_default())
        .collect();

    let c_env = shell.env.as_execve_list();
    reset_child_signals();
    match execve(&c_path, &c_args, &c_env) {
        Err(EACCES) => {
            eprintln!("seal: {program} Permission denied");
            exit(126)
        }
        Err(e) => {
            eprintln!("seal: {program} {e}");
            exit(126)
        }
    }
}

fn resolve_binary_path(shell: &mut Shell, program: &String) -> Option<PathBuf> {
    if program.contains('/') {
        let path = PathBuf::from(program);
        if path.is_executable() {
            return Some(path);
        } else {
            return None;
        }
    }
    let path_var = shell.env.get("PATH").unwrap_or("/bin:/usr/bin:/usr/sbin");
    for dir in path_var.split(":") {
        let candidate = PathBuf::from(dir).join(program);
        if candidate.is_executable() {
            return Some(candidate);
        }
    }
    None
}

fn reset_child_signals() {
    unsafe {
        let _ = signal(Signal::SIGINT, SigHandler::SigDfl);
        let _ = signal(Signal::SIGQUIT, SigHandler::SigDfl);
        let _ = signal(Signal::SIGTSTP, SigHandler::SigDfl);
        let _ = signal(Signal::SIGTTIN, SigHandler::SigDfl);
        let _ = signal(Signal::SIGTTOU, SigHandler::SigDfl);
        let _ = signal(Signal::SIGCHLD, SigHandler::SigDfl);
    }
}

fn run_builtin_with_redirects(shell: &mut Shell, cmd: &UnitCommand) -> i32 {
    let saved = match save_std_fds() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("seal: {e}");
            return 1;
        }
    };

    let exit_code = match apply_redirects(&cmd.redirects) {
        Ok(_) => run_builtin(shell, &cmd.words).unwrap_or(127),
        Err(e) => {
            eprintln!("seal: {e}");
            1
        }
    };
    let _ = std::io::stdout().flush();
    restore_std_fds(&saved);
    exit_code
}

fn save_std_fds() -> nix::Result<[OwnedFd; 3]> {
    let dup_high = |fd: RawFd| -> nix::Result<OwnedFd> {
        let borrowed = unsafe { BorrowedFd::borrow_raw(fd) };
        let new = nix::fcntl::fcntl(borrowed, FcntlArg::F_DUPFD_CLOEXEC(10))?;
        Ok(unsafe { OwnedFd::from_raw_fd(new) })
    };
    Ok([dup_high(0)?, dup_high(1)?, dup_high(2)?])
}
fn restore_std_fds(saved: &[OwnedFd; 3]) {
    for (target, fd) in saved.iter().enumerate() {
        let _ = unsafe { dup2_raw(fd, target as RawFd).map(std::mem::forget) };
    }
}

fn is_builtin(shell: &Shell, cmd: &UnitCommand) -> bool {
    if cmd.words.is_empty() {
        false
    } else {
        shell.builtins.get(&cmd.words[0]).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::node::{RedirectKind, RedirectTarget, UnitCommand};

    fn unit(words: &[&str]) -> Command {
        Command::Unit(UnitCommand {
            words: words.iter().map(|&s| s.to_string()).collect(),
            redirects: Vec::new(),
        })
    }

    fn unit_redir(words: &[&str], redirects: Vec<Redirect>) -> Command {
        Command::Unit(UnitCommand {
            words: words.iter().map(|&s| s.to_string()).collect(),
            redirects,
        })
    }

    // ================= 1. Single Command Tests =================

    #[test]
    fn test_single_command_success() {
        let mut shell = Shell::new();
        let cmd = unit(&["true"]);
        let code = execute_pipeline(&mut shell, &[cmd], false);
        assert_eq!(code, 0);
    }

    #[test]
    fn test_single_command_failure() {
        let mut shell = Shell::new();
        let cmd = unit(&["false"]);
        let code = execute_pipeline(&mut shell, &[cmd], false);
        assert_eq!(code, 1);
    }

    #[test]
    fn test_single_command_with_args() {
        let mut shell = Shell::new();
        let cmd = unit(&["echo", "hello", "seal"]);
        let code = execute_pipeline(&mut shell, &[cmd], false);
        assert_eq!(code, 0);
    }

    #[test]
    fn test_single_command_not_found() {
        let mut shell = Shell::new();
        let cmd = unit(&["__non_existent_command_seal_xyz_123__"]);
        let code = execute_pipeline(&mut shell, &[cmd], false);
        assert_eq!(code, 127);
    }

    // ================= 2. Pipeline Tests =================

    #[test]
    fn test_pipeline_two_commands_success() {
        // echo hello | grep hello -> 0
        let mut shell = Shell::new();
        let cmds = vec![unit(&["echo", "hello"]), unit(&["grep", "hello"])];
        let code = execute_pipeline(&mut shell, &cmds, false);
        assert_eq!(code, 0);
    }

    #[test]
    fn test_pipeline_grep_no_match() {
        // echo hello | grep world -> 1 (grep returns 1 when pattern not found)
        let mut shell = Shell::new();
        let cmds = vec![unit(&["echo", "hello"]), unit(&["grep", "world"])];
        let code = execute_pipeline(&mut shell, &cmds, false);
        assert_eq!(code, 1);
    }

    #[test]
    fn test_pipeline_exit_code_is_last_command_success() {
        // false | true -> 0 (POSIX rule: pipeline code is from the last command)
        let mut shell = Shell::new();
        let cmds = vec![unit(&["false"]), unit(&["true"])];
        let code = execute_pipeline(&mut shell, &cmds, false);
        assert_eq!(code, 0);
    }

    #[test]
    fn test_pipeline_exit_code_is_last_command_failure() {
        // true | false -> 1 (POSIX rule)
        let mut shell = Shell::new();
        let cmds = vec![unit(&["true"]), unit(&["false"])];
        let code = execute_pipeline(&mut shell, &cmds, false);
        assert_eq!(code, 1);
    }

    #[test]
    fn test_pipeline_three_stages() {
        // echo "apple" | tr a-z A-Z | grep APPLE -> 0
        let mut shell = Shell::new();
        let cmds = vec![
            unit(&["echo", "apple"]),
            unit(&["tr", "a-z", "A-Z"]),
            unit(&["grep", "APPLE"]),
        ];
        let code = execute_pipeline(&mut shell, &cmds, false);
        assert_eq!(code, 0);
    }

    // ================= 3. Redirection Tests =================

    #[test]
    fn test_redirection_output_and_input() {
        let temp_file = "/tmp/seal_test_output_input.txt";
        let _ = std::fs::remove_file(temp_file);

        let mut shell = Shell::new();

        // echo seal_magic_token > temp_file
        let redir_out = Redirect {
            fd: None,
            kind: RedirectKind::Output,
            target: RedirectTarget::File(temp_file.to_string()),
        };
        let write_cmd = unit_redir(&["echo", "seal_magic_token"], vec![redir_out]);
        let code1 = execute_pipeline(&mut shell, &[write_cmd], false);
        assert_eq!(code1, 0);

        // grep seal_magic_token < temp_file
        let redir_in = Redirect {
            fd: None,
            kind: RedirectKind::Input,
            target: RedirectTarget::File(temp_file.to_string()),
        };
        let read_cmd = unit_redir(&["grep", "seal_magic_token"], vec![redir_in]);
        let code2 = execute_pipeline(&mut shell, &[read_cmd], false);
        assert_eq!(code2, 0);

        let _ = std::fs::remove_file(temp_file);
    }

    #[test]
    fn test_redirection_append() {
        let temp_file = "/tmp/seal_test_append.txt";
        let _ = std::fs::remove_file(temp_file);

        let mut shell = Shell::new();

        // echo line1 > temp_file
        let redir_out = Redirect {
            fd: None,
            kind: RedirectKind::Output,
            target: RedirectTarget::File(temp_file.to_string()),
        };
        execute_pipeline(
            &mut shell,
            &[unit_redir(&["echo", "line1"], vec![redir_out])],
            false,
        );

        // echo line2 >> temp_file
        let redir_append = Redirect {
            fd: None,
            kind: RedirectKind::Append,
            target: RedirectTarget::File(temp_file.to_string()),
        };
        execute_pipeline(
            &mut shell,
            &[unit_redir(&["echo", "line2"], vec![redir_append])],
            false,
        );

        // grep -c line < temp_file | grep 2
        let redir_in = Redirect {
            fd: None,
            kind: RedirectKind::Input,
            target: RedirectTarget::File(temp_file.to_string()),
        };
        let cmds = vec![
            unit_redir(&["grep", "-c", "line"], vec![redir_in]),
            unit(&["grep", "2"]),
        ];
        let code = execute_pipeline(&mut shell, &cmds, false);
        assert_eq!(code, 0);

        let _ = std::fs::remove_file(temp_file);
    }

    #[test]
    fn test_redirection_input_missing_file_fails() {
        let mut shell = Shell::new();
        let redir_in = Redirect {
            fd: None,
            kind: RedirectKind::Input,
            target: RedirectTarget::File("/tmp/__seal_missing_file_987654321__.txt".to_string()),
        };
        let cmd = unit_redir(&["cat"], vec![redir_in]);
        let code = execute_pipeline(&mut shell, &[cmd], false);
        assert_eq!(code, 1);
    }

    // ================= 4. Control Flow (execute AST) Tests =================

    #[test]
    fn test_logical_and_short_circuit() {
        let mut shell = Shell::new();

        // true && true -> 0
        let node_true = SyntaxNode::And {
            left: Box::new(SyntaxNode::Pipeline(vec![unit(&["true"])])),
            right: Box::new(SyntaxNode::Pipeline(vec![unit(&["true"])])),
        };
        assert_eq!(execute(&mut shell, &node_true), 0);

        // false && true -> 1 (short-circuited, second not run)
        let node_false = SyntaxNode::And {
            left: Box::new(SyntaxNode::Pipeline(vec![unit(&["false"])])),
            right: Box::new(SyntaxNode::Pipeline(vec![unit(&["true"])])),
        };
        assert_eq!(execute(&mut shell, &node_false), 1);
    }

    #[test]
    fn test_logical_or_short_circuit() {
        let mut shell = Shell::new();

        // true || false -> 0 (short-circuited, second not run)
        let node_or1 = SyntaxNode::Or {
            left: Box::new(SyntaxNode::Pipeline(vec![unit(&["true"])])),
            right: Box::new(SyntaxNode::Pipeline(vec![unit(&["false"])])),
        };
        assert_eq!(execute(&mut shell, &node_or1), 0);

        // false || true -> 0
        let node_or2 = SyntaxNode::Or {
            left: Box::new(SyntaxNode::Pipeline(vec![unit(&["false"])])),
            right: Box::new(SyntaxNode::Pipeline(vec![unit(&["true"])])),
        };
        assert_eq!(execute(&mut shell, &node_or2), 0);
    }

    #[test]
    fn test_sequence_exit_code() {
        let mut shell = Shell::new();

        // false ; true -> 0 (returns last exit code)
        let seq1 = SyntaxNode::Sequence(vec![
            SyntaxNode::Pipeline(vec![unit(&["false"])]),
            SyntaxNode::Pipeline(vec![unit(&["true"])]),
        ]);
        assert_eq!(execute(&mut shell, &seq1), 0);

        // true ; false -> 1 (returns last exit code)
        let seq2 = SyntaxNode::Sequence(vec![
            SyntaxNode::Pipeline(vec![unit(&["true"])]),
            SyntaxNode::Pipeline(vec![unit(&["false"])]),
        ]);
        assert_eq!(execute(&mut shell, &seq2), 1);
    }
}
