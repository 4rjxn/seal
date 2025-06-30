use std::io;
use std::io::Write;
use std::process;
use std::str::SplitWhitespace;
use std::path::Path;
use std::env;

fn main() {
    loop{
        let mut input = String::new();
        print!(":>");
        io::stdout().flush().expect("flush error");
        io::stdin().read_line(&mut input).expect("read error");
        let mut parts = input.trim().split_whitespace();
        let command = match parts.next(){
            Some(rslt)=>rslt.to_string(),
            None =>continue,
        };
        let args = parts;
        match command.as_str() {
            "exit"=>process::exit(0x0100),
            "cd" => {
                let new_dir = args.peekable().peek().map_or("/", |x| *x);
                let root = Path::new(new_dir);
                env::set_current_dir(&root).unwrap();
            }
            _=>execute(&command,args),
        }
    }
}

fn execute<'a>(command:&'a String,args:SplitWhitespace){
    let mut child = match process::Command::new(command).args(args).spawn(){
        Ok(rst)=>rst,
        Err(_)=>{
            println!("'{}' command not found",command);
            return;
        },
    };
    child.wait().unwrap();
}
