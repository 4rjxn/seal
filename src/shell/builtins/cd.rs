use std::{env::set_current_dir, path::PathBuf};

use crate::shell::{self, Shell, builtins::BuiltIn};

pub struct Cd;

impl BuiltIn for Cd {
    fn name(&self) -> &str {
        "cd"
    }

    fn help(&self) -> &str {
        "Usage: cd [path]\n\
        \n\
         change current directory to the given path\n\
         if no path is given it will be changed to $HOME\n\
         if '-' is given as path it will be changed to $OLDPWD"
    }

    fn run(&self, ctx: &mut Shell, args: &[String]) -> i32 {
        if args.len() > 2 {
            eprintln!("seal: cd: too many arguments");
            return 1;
        }
        let mut print_new = false;
        let target = match args.get(1).map(String::as_str) {
            None => match ctx.env.get("HOME") {
                Some(h) => h.to_string(),
                None => {
                    eprintln!("seal: cd: HOME not set");
                    return 1;
                }
            },
            Some("-") => match ctx.env.get("OLDPWD") {
                Some(old) => {
                    print_new = true;
                    old.to_string()
                }
                None => {
                    eprintln!("seal: cd: OLDPWD not set");
                    return 1;
                }
            },
            Some(p) => p.to_string(),
        };

        let old = std::env::current_dir().ok();

        if let Err(e) = set_current_dir(&target) {
            let err = nix::errno::Errno::from_raw(e.raw_os_error().unwrap_or(0)).desc();
            eprintln!("seal: cd: {target}: {err}");
            return 1;
        }
        let new = std::env::current_dir().unwrap_or_else(|_| PathBuf::from(&target));

        if let Some(old) = old {
            ctx.env.set("OLDPWD", old);
        }
        ctx.env.set("PWD", &new);
        ctx.env.set_cwd(new.clone());
        if print_new {
            println!("{}", new.display());
        }
        0
    }
}
