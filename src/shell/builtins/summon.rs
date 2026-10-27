use std::{io::Write, thread::sleep, time::Duration};

use crate::shell::{Shell, builtins::BuiltIn};

pub struct Summon;

const RED: &str = "\x1b[31m";
const BOLD_RED: &str = "\x1b[1;31m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

const BANNER_TOP: &str = r"
   ⛧ ─────────────────────────────────────── ⛧
            SIC MUNDUS   CREATUS EST
   ⛧ ─────────────────────────────────────── ⛧
";

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

const BANNER_BOTTOM: &str = r"
   ⛧  ᚦ  ᛉ  ᛟ  ᚨ  ᛞ  ᚱ  ᛊ  ᛏ  ᚲ  ᛗ  ᚹ  ⛧
";

impl BuiltIn for Summon {
    fn name(&self) -> &str {
        "summon"
    }

    fn help(&self) -> &str {
        "Usage: summon\n\
         \n\
         Display the Seal of Solomon. (eye candy)"
    }

    fn run(&self, _ctx: &mut Shell, _args: &[String]) -> i32 {
        print!("{BOLD_RED}{BANNER_TOP}{RESET}");
        for line in SEAL.lines() {
            println!("{RED}{line}{RESET}");
            let _ = std::io::stdout().flush();
            sleep(Duration::from_millis(70));
        }
        print!("{DIM}{RED}{BANNER_BOTTOM}{RESET}");
        println!("{BOLD_RED}      \"By seal and name, I bind thee.\"{RESET}\n");
        0
    }
}
