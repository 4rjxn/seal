use std::{io::stdin, path::PathBuf, process::exit};

use nix::{
    libc::{STDIN_FILENO, tcsetpgrp},
    sys::signal::{SigHandler, Signal, signal},
    unistd::{Pid, isatty, setpgid},
};
use seal::{
    lua::LuaEngine,
    shell::{Shell, job_control::reap_background, opts::ShellOpts},
};

use crate::{repl::Repl, version::print_version};
mod prompt;
mod repl;
mod version;

fn main() {
    handle_args();
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

    let engine = LuaEngine::new().ok();
    let config_path = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .map(|p| p.join("seal/init.lua"));

    match &engine {
        Some(e) => {
            if let Some(path) = config_path {
                if let Err(_) = e.run_luafile(&mut shell, path) {
                    eprintln!("seal: running without any config");
                };
            }
        }
        None => {}
    }

    let mut repl = Repl::new();
    loop {
        if let Some(str) = repl.read() {
            for msg in reap_background(&mut shell.jobs) {
                eprintln!("\n{}", msg)
            }
            if str.trim().is_empty() {
                continue;
            }
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

fn handle_args() {
    let args: Vec<String> = std::env::args().collect();
    if matches!(
        args.get(1).map(String::as_str),
        Some("--version" | "-v" | "-V")
    ) {
        print_version();
        exit(0);
    }
}
