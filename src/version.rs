use std::{
    io::{IsTerminal, Write},
    thread::sleep,
    time::Duration,
};

const RED: &str = "\x1b[31m";
const BOLD_RED: &str = "\x1b[1;31m";
const DIM_RED: &str = "\x1b[2;31m";
const RESET: &str = "\x1b[0m";

const VERSION: &str = env!("CARGO_PKG_VERSION");

const SEAL: &str = r"
                   /\
                  /  \
            _____/____\_____
           \    /      \    /
            \  /        \  /
             \/          \/
             /\          /\
            /  \        /  \
           /____\______/____\
                 \    /
                  \  /
                   \/
";

pub fn print_version() {
    let tty = std::io::stdout().is_terminal();

    if !tty {
        // plain output when piped: `seal --version | cat`
        println!("seal {VERSION}");
        return;
    }

    println!("{BOLD_RED}   ⛧ ─────────────────────────────── ⛧{RESET}");
    println!("{BOLD_RED}          S   E   A   L   {DIM_RED}v{VERSION}{RESET}");
    println!("{BOLD_RED}   ⛧ ─────────────────────────────── ⛧{RESET}");

    for line in SEAL.lines() {
        println!("{RED}{line}{RESET}");
        let _ = std::io::stdout().flush();
        sleep(Duration::from_millis(60));
    }
}
