use std::{env, path::PathBuf};

use is_executable::IsExecutable;
use rustyline::{DefaultEditor, error::ReadlineError};

#[derive(Debug)]
pub enum Tokens {
    Word(String),
    Pipe,
    Output,
    Append,
    OutputErr,
    AppendErr,
}

#[derive(Debug)]
pub enum Redirects {
    //Input(String),
    Output(String),
    OutputErr(String),
    Append(String),
    AppendErr(String),
}

#[derive(Debug)]
pub struct Command {
    pub program: String,
    pub args: Vec<String>,
    pub redirects: Vec<Redirects>,
}

pub fn locate_command(command: &String) -> Result<PathBuf, ()> {
    match env::var_os("PATH") {
        Some(paths) => {
            for mut path in env::split_paths(&paths) {
                path = path.join(command);
                if path.is_executable() {
                    return Ok(path);
                }
            }
            return Err(());
        }
        None => Err(()),
    }
}

fn is_valid_command(command: &String) -> bool {
    match locate_command(command) {
        Ok(_) => return true,
        Err(_) => return false,
    }
}

pub enum Builtins {
    Echo,
    Exit,
    Type,
    Pwd,
    Cd,
}

pub fn is_builtin(command: &String) -> Result<Builtins, ()> {
    return match command.as_str() {
        "exit" => Ok(Builtins::Exit),
        "echo" => Ok(Builtins::Echo),
        "type" => Ok(Builtins::Type),
        "pwd" => Ok(Builtins::Pwd),
        "cd" => Ok(Builtins::Cd),
        _ => Err(()),
    };
}

impl Command {
    pub fn is_valid(&self) -> bool {
        is_builtin(&self.program).is_ok() || is_valid_command(&self.program)
    }
    pub fn is_builtin(&self) -> Result<Builtins, ()> {
        is_builtin(&self.program)
    }
}

#[derive(Debug)]
pub struct Pipeline {
    pub commands: Vec<Command>,
}

fn string_to_token(val: &String) -> Tokens {
    match val.as_str() {
        "2>" => Tokens::OutputErr,
        "2>>" => Tokens::AppendErr,
        ">>" | "1>>" => Tokens::Append,
        ">" | "1>" => Tokens::Output,
        "|" => Tokens::Pipe,
        _ => Tokens::Word(val.to_owned()),
    }
}

pub fn read_and_parse() -> Vec<Tokens> {
    let mut rl = DefaultEditor::new().unwrap();
    loop {
        let command_input;
        let readline = rl.readline(String::from(">> ").as_str());
        match readline {
            Ok(line) => {
                command_input = line;
            }
            Err(ReadlineError::Interrupted) => {
                continue;
            }
            Err(ReadlineError::Eof) => continue,
            Err(err) => {
                eprintln!("Error: {:?}", err);
                continue;
            }
        }
        let mut tokens = Vec::new();
        let mut word = String::new();
        let mut is_quote = false;
        let mut is_double_quote = false;
        let mut is_black_slash = false;
        if command_input.is_empty() {
            continue;
        }
        for char in command_input.trim().chars() {
            if is_black_slash {
                word.push(char);
                is_black_slash = !is_black_slash;
                continue;
            }
            match char {
                '\\' => {
                    if !is_quote {
                        is_black_slash = !is_black_slash;
                        continue;
                    }
                    word.push(char);
                }
                '"' => {
                    if !is_quote {
                        is_double_quote = !is_double_quote;
                        continue;
                    }
                    word.push(char);
                }
                '\'' => {
                    if !is_double_quote {
                        is_quote = !is_quote;
                        continue;
                    }
                    word.push(char);
                }
                ' ' => {
                    if is_quote || is_double_quote {
                        word.push(char);
                        continue;
                    }
                    if !word.is_empty() {
                        tokens.push(string_to_token(&word));
                        word = String::new();
                    }
                }
                _ => word.push(char),
            }
        }
        if !word.is_empty() {
            tokens.push(string_to_token(&word));
        }
        return tokens;
    }
}

fn parse_command(tokens: &mut Vec<Tokens>) -> Option<Command> {
    let mut program = None;
    let mut args = Vec::new();
    let mut redirects = Vec::new();
    while let Some(token) = tokens.first() {
        match token {
            Tokens::Word(word) => {
                let word = word.to_owned();
                tokens.remove(0);
                if program.is_none() {
                    if let Ok(program_path) = locate_command(&word) {
                        program = Some(program_path.to_str().unwrap().to_string());
                        args.push(word);
                    } else {
                        program = Some(word.to_owned());
                        args.push(word);
                    }
                } else {
                    args.push(word);
                }
            }
            Tokens::Output => {
                tokens.remove(0);
                if let Some(Tokens::Word(file)) = tokens.first() {
                    redirects.push(Redirects::Output(file.to_owned()));
                    tokens.remove(0);
                }
            }
            Tokens::OutputErr => {
                tokens.remove(0);
                if let Some(Tokens::Word(file)) = tokens.first() {
                    redirects.push(Redirects::OutputErr(file.to_owned()));
                    tokens.remove(0);
                }
            }
            Tokens::Append => {
                tokens.remove(0);
                if let Some(Tokens::Word(file)) = tokens.first() {
                    redirects.push(Redirects::Append(file.to_owned()));
                    tokens.remove(0);
                }
            }
            Tokens::AppendErr => {
                tokens.remove(0);
                if let Some(Tokens::Word(file)) = tokens.first() {
                    redirects.push(Redirects::AppendErr(file.to_owned()));
                    tokens.remove(0);
                }
            }
            Tokens::Pipe => break,
        }
    }
    return Some(Command {
        program: program?,
        args,
        redirects,
    });
}

pub fn parse_pipelines(tokens: Vec<Tokens>) -> Option<Pipeline> {
    let mut tokens = tokens;
    let mut commands = Vec::new();
    while !tokens.is_empty() {
        let cmd = parse_command(&mut tokens)?;
        commands.push(cmd);
        if matches!(tokens.first(), Some(Tokens::Pipe)) {
            tokens.remove(0);
        } else {
            break;
        }
    }
    Some(Pipeline { commands })
}
