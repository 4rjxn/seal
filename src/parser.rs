use std::io::{Write, stdin, stdout};

#[derive(Debug)]
pub enum Tokens {
    Word(String),
    Pipe,
    Output,
    Append,
}

#[derive(Debug)]
enum Redirects {
    //Input(String),
    Output(String),
    Append(String),
}

#[derive(Debug)]
pub struct Command {
    program: String,
    args: Vec<String>,
    redirects: Vec<Redirects>,
}

#[derive(Debug)]
pub struct Pipeline {
    commands: Vec<Command>,
}

fn string_to_token(val: &String) -> Tokens {
    match val.as_str() {
        ">>" => Tokens::Append,
        ">" => Tokens::Output,
        "|" => Tokens::Pipe,
        _ => Tokens::Word(val.to_owned()),
    }
}

pub fn read_and_parse() -> Vec<Tokens> {
    let mut command_input = String::new();
    print!("$ ");
    stdout().flush().unwrap();
    stdin().read_line(&mut command_input).unwrap();
    let mut tokens = Vec::new();
    let mut word = String::new();
    let mut is_quote = false;
    let mut is_double_quote = false;
    let mut is_black_slash = false;
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
    tokens
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
                    program = Some(word)
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
            Tokens::Append => {
                tokens.remove(0);
                if let Some(Tokens::Word(file)) = tokens.first() {
                    redirects.push(Redirects::Append(file.to_owned()));
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
