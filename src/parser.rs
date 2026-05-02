use std::{env, path::PathBuf};

use is_executable::IsExecutable;

use crate::{
    error::{ShellError, ShellResult},
    models::{Command, Redirect, RedirectKind, Token},
    traits::Expantions,
};

struct TokenStream {
    tokens: Vec<Token>,
    pos: usize,
}

impl TokenStream {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }
    fn next(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.pos)?;
        self.pos += 1;
        Some(token)
    }

    fn expect_word(&mut self) -> Option<String> {
        match self.peek() {
            Some(Token::Word { value, .. }) => {
                let word = value.clone();
                self.pos += 1;
                Some(word)
            }
            _ => None,
        }
    }

    fn is_done(&self) -> bool {
        self.pos >= self.tokens.len()
    }
}

fn parse_pipeline(tokens: Vec<Token>) -> Vec<Command> {
    let mut stream = TokenStream::new(tokens);
    let mut commands = Vec::new();
    loop {
        if let Some(cmd) = parse_command(&mut stream) {
            commands.push(cmd);
        }

        match stream.peek() {
            Some(Token::Pipe) => stream.next(),
            _ => break,
        };
    }
    commands
}

fn parse_command(stream: &mut TokenStream) -> Option<Command> {
    let (program, first_arg) = parse_program(stream)?;
    let (args, redirects) = parse_args_and_redirects(stream, first_arg);
    Some(Command {
        program,
        args,
        redirects,
    })
}

fn parse_args_and_redirects(
    stream: &mut TokenStream,
    first_arg: String,
) -> (Vec<String>, Vec<Redirect>) {
    let mut args = vec![first_arg];
    let mut redirects = Vec::new();

    while let Some(token) = stream.peek() {
        match token {
            Token::Word { .. } => parse_arg(stream, &mut args),
            Token::Output | Token::OutputErr | Token::Append | Token::AppendErr => {
                parse_redirect(stream, &mut redirects)
            }
            Token::Pipe | Token::Background => break,
        }
    }
    (args, redirects)
}

fn parse_arg(stream: &mut TokenStream, args: &mut Vec<String>) {
    if let Some(Token::Word { value, quoted }) = stream.next() {
        if *quoted {
            args.push(value.clone());
        } else {
            let expanded = value.expand_path().expand_glob();
            args.extend(expanded);
        }
    }
}

fn parse_redirect(stream: &mut TokenStream, redirects: &mut Vec<Redirect>) {
    let kind = match stream.next() {
        Some(Token::Output) => RedirectKind::Output,
        Some(Token::OutputErr) => RedirectKind::OutputErr,
        Some(Token::Append) => RedirectKind::Append,
        Some(Token::AppendErr) => RedirectKind::AppendErr,
        _ => return,
    };
    if let Some(target) = stream.expect_word() {
        redirects.push(Redirect { kind, target });
    } else {
        eprintln!("syntax error: expected filename after redirect");
    }
}

fn parse_program(stream: &mut TokenStream) -> Option<(String, String)> {
    let word = stream.expect_word()?;
    let path = locate_command(&word)
        .ok()
        .and_then(|p| p.to_str().map(str::to_string))
        .unwrap_or_else(|| word.clone());
    Some((path, word))
}

pub fn locate_command(command: &String) -> ShellResult<PathBuf> {
    if command.starts_with("./") {
        return current_absolute_path(command);
    }
    get_path_from_env(command)
}

fn get_path_from_env(command: &String) -> ShellResult<PathBuf> {
    match env::var_os("PATH") {
        Some(paths) => {
            for mut path in env::split_paths(&paths) {
                path = path.join(command);
                if path.is_executable() {
                    return Ok(path);
                }
            }
            return Err(ShellError::CommandNotFound(command.clone()));
        }
        None => Err(ShellError::CommandNotFound(command.clone())),
    }
}

fn current_absolute_path(command: &String) -> ShellResult<PathBuf> {
    let command = command.replace("./", "");
    let curr_dir = env::current_dir().unwrap();
    return Ok(PathBuf::from(curr_dir).join(command));
}
