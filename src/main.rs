use std::io;
use std::io::Write;
use std::process;
use std::path::Path;
use std::env;
use std::fs;


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
    match env::set_current_dir(path){
        Ok(()) => {},
        Err(err) => eprintln!("{}",err),
    }
    essentials
}

fn tokenise(input:&String) -> Vec<String>{
    let chr_list = input.chars();
    let mut read_buff:String = String::new();
    let mut argv:Vec<String> = Vec::new();
    let mut is_quote:bool = false;
    for i in chr_list{
        if i == '"'{
            is_quote = !is_quote;
            continue;
        }
        if !is_quote && i == ' ' || i == '\n'{
            argv.push(read_buff);
            read_buff = String::new();
        }
        else{
            read_buff.push(i);
        }
    }
    argv
}

fn main() {
    let user_details = set_env();
    loop{
        let mut input = String::new();
        print!("{}@{}:",user_details.user,user_details.host);
        io::stdout().flush().expect("flush error");
        io::stdin().read_line(&mut input).expect("read error");
        let argv = tokenise(&input);
        match argv[0].as_str() {
            "exit"=>process::exit(0x0100),
            "cd" => {
                let new_dir = if argv.len()>1 {&argv[1]} else {&user_details.home};
                let root = Path::new(&new_dir);
                match env::set_current_dir(root){
                    Ok(()) => {},
                    Err(err) => eprintln!("{}",err),
                }
            }
            _=>execute(argv),
        }
    }
}

fn execute(argv:Vec<String>){
    let mut child = match process::Command::new(&argv[0]).args(&argv[1..]).spawn(){
        Ok(rst)=>rst,
        Err(_)=>{
            println!("'{}' command not found",argv[0]);
            return;
        },
    };
    child.wait().unwrap();
}
