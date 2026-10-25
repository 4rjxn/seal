use std::{io::stdin, process::exit};

use nix::{
    libc::{STDIN_FILENO, tcsetpgrp},
    sys::signal::{SigHandler, Signal, signal},
    unistd::{Pid, isatty, setpgid},
};
use seal::shell::{Shell, job_control::reap_background, opts::ShellOpts};

use crate::repl::Repl;
mod prompt;
mod repl;

fn main() {
    set_signals_for_parent();

    // Make the shell the leader of its own process group before claiming the
    // terminal. Without this, `Pid::this()` is not a valid pgid when the shell
    // was not started by a job-control shell (e.g. `cargo run`), and the
    // `tcsetpgrp` below would attach the terminal to the wrong group.
    let _ = setpgid(Pid::from_raw(0), Pid::from_raw(0));

    let mut shell = Shell::with_opts(ShellOpts {
        is_interactive: true,
    });
    if isatty(stdin()).unwrap_or(false)
        && unsafe { tcsetpgrp(STDIN_FILENO, shell.shell_pgid.as_raw()) } != 0
    {
        eprintln!(
            "seal: failed to take control of the terminal: {}",
            std::io::Error::last_os_error()
        );
    }
    let mut repl = Repl::new();
    loop {
        for msg in reap_background(&mut shell.jobs) {
            eprintln!("\n{}", msg)
        }
        if let Some(str) = repl.read() {
            let _ = shell.execute_str(&str);
        } else {
            exit(shell.last_exit)
        }
    }
}

fn set_signals_for_parent() {
    unsafe {
        signal(Signal::SIGTSTP, SigHandler::SigIgn).unwrap();
        signal(Signal::SIGTTOU, SigHandler::SigIgn).unwrap();
        signal(Signal::SIGTTIN, SigHandler::SigIgn).unwrap();
        signal(Signal::SIGINT, SigHandler::SigIgn).unwrap();
    }
}
