use std::io;
use std::io::Write;
use std::process::Command;
use std::str::SplitWhitespace;

fn main() {
    loop{
        let mut input = String::new();
        print!(":");
        io::stdout().flush().expect("");
        io::stdin().read_line(&mut input).expect("haii");
        let mut parts = input.trim().split_whitespace();
        let command = parts.next().unwrap().to_string();
        let args = parts;
        execute(&command,args);
    }
}

fn execute<'a>(command:&'a String,args:SplitWhitespace){
    let mut child = Command::new(command).args(args).spawn().unwrap();
    child.wait().unwrap();
}
