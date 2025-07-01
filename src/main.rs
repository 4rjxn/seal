use std::io;
use std::io::Write;
use std::process;
use std::str::SplitWhitespace;
use std::path::Path;
use std::env;
use std::fs;

struct UserInput<'b>{
    command:String,
    args:SplitWhitespace<'b>,
}

struct RunEssentials{
    host:String,
    user:String,
    home:String,
}

impl RunEssentials{
    fn set_values() -> RunEssentials{
        RunEssentials{
            host:fs::read_to_string("/etc/hostname").expect("err").trim().to_string(),
            user: match env::var_os("USER"){
                Some(ostr) => ostr.into_string().expect("err"),
                None => panic!(),
            },
            home:match env::var_os("HOME"){
                Some(ostr) => ostr.into_string().expect("err"),
                None => String::from("/"),
            },
        }
    }
}

fn set_env()->RunEssentials{
    let essentials = RunEssentials::set_values();
    let path = Path::new(&essentials.home);
    env::set_current_dir(path).unwrap();
    essentials
}

fn tokenise(input:&String){
    let chr_list = input.as_bytes();
    let mut is_quote:bool = false;
    let mut list:Vec<&str> = Vec::new();
    let mut strbuffer:Vec<&str> = Vec::new();
    let mut index = 0;
    for (i,&val) in chr_list.iter().enumerate(){
        if (val == b' ' || val == b'"') && !is_quote{
            list.push(&input[index..i]);
            index = i+1;
        }
        if (val == b' ' || val == b'"') && is_quote{
            strbuffer.push(&input[index..i]);
            index = i+1;
        }
        if val == b'"'{
            is_quote = !is_quote;
        }
    }
    println!("{:?}",list);
    println!("{:?}",strbuffer.join(""));
}

fn main() {
    let user_details = set_env();
    //println!("\"hai helloo ahd\"");
    loop{
        let mut input = String::new();
        print!("{}@{}:",user_details.user,user_details.host);
        io::stdout().flush().expect("flush error");
        io::stdin().read_line(&mut input).expect("read error");
        tokenise(&input);
        let mut parts = input.split_whitespace();
        let user_input = UserInput{
            command:match parts.next(){
                Some(rslt)=>rslt.to_string(),
                None =>continue,
            },
            args:parts,
        };
        match user_input.command.as_str() {
            "exit"=>process::exit(0x0100),
            "cd" => {
                let new_dir = user_input.args.peekable().peek().map_or("/", |x| *x);
                let root = Path::new(new_dir);
                env::set_current_dir(root).unwrap();
            }
            _=>execute(user_input),
        }
    }
}

fn execute(user_input:UserInput){
    let mut child = match process::Command::new(&user_input.command).args(user_input.args).spawn(){
        Ok(rst)=>rst,
        Err(_)=>{
            println!("'{}' command not found",user_input.command);
            return;
        },
    };
    child.wait().unwrap();
}
