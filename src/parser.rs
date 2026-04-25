use std::{env, path::PathBuf};

use is_executable::IsExecutable;

use crate::error::ShellError;
use crate::error::ShellResult;
use crate::models::Builtins;
use crate::models::Command;
use crate::models::Pipeline;
use crate::models::Redirects;
use crate::models::Tokens;
use crate::traits::Expantions;

pub fn locate_command(command: &String) -> ShellResult<PathBuf> {
    if command.starts_with("./") {
        let command = command.replace("./", "");
        let curr_dir = env::current_dir().unwrap();
        return Ok(PathBuf::from(curr_dir).join(command));
    }
    if let Ok(_) = is_builtin(command) {
        return Ok(PathBuf::from(command));
    }
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

pub fn is_valid_command(command: &String) -> bool {
    locate_command(command).is_ok()
}

pub fn is_builtin(command: &String) -> ShellResult<Builtins> {
    match command.as_str() {
        "exit" => Ok(Builtins::Exit),
        "echo" => Ok(Builtins::Echo),
        "type" => Ok(Builtins::Type),
        "pwd" => Ok(Builtins::Pwd),
        "cd" => Ok(Builtins::Cd),
        "fg" => Ok(Builtins::Fg),
        "jobs" => Ok(Builtins::Jobs),
        _ => Err(ShellError::InvalidBuiltin(command.clone())),
    }
}

fn parse_command(tokens: &mut Vec<Tokens>) -> Option<Command> {
    let mut program = None;
    let mut args = Vec::new();
    let mut redirects = Vec::new();
    while let Some(token) = tokens.first() {
        match token {
            Tokens::Word { value, quoted } => {
                let word = value.to_owned();
                if program.is_none() {
                    let program_path = locate_command(&word)
                        .ok()
                        .and_then(|p| p.to_str().map(|s| s.to_string()))
                        .unwrap_or_else(|| word.clone());
                    program = Some(program_path);
                    args.push(word);
                    tokens.remove(0);
                    continue;
                }

                match quoted {
                    false => {
                        let word = word.expand_path();
                        let word = word.expand_glob();
                        args.extend(word);
                    }
                    true => {
                        args.push(word);
                    }
                }
                tokens.remove(0);
            }
            Tokens::Output => {
                tokens.remove(0);
                if let Some(Tokens::Word { value, quoted: _ }) = tokens.first() {
                    redirects.push(Redirects::Output(value.to_owned()));
                    tokens.remove(0);
                }
            }
            Tokens::OutputErr => {
                tokens.remove(0);
                if let Some(Tokens::Word { value, quoted: _ }) = tokens.first() {
                    redirects.push(Redirects::OutputErr(value.to_owned()));
                    tokens.remove(0);
                }
            }
            Tokens::Append => {
                tokens.remove(0);
                if let Some(Tokens::Word { value, quoted: _ }) = tokens.first() {
                    redirects.push(Redirects::Append(value.to_owned()));
                    tokens.remove(0);
                }
            }
            Tokens::AppendErr => {
                tokens.remove(0);
                if let Some(Tokens::Word { value, quoted: _ }) = tokens.first() {
                    redirects.push(Redirects::AppendErr(value.to_owned()));
                    tokens.remove(0);
                }
            }
            Tokens::Pipe => break,
            Tokens::Background => {}
        }
    }
    return Some(Command {
        program: program?,
        args,
        redirects,
    });
}

fn is_background(tokens: &mut Vec<Tokens>) -> bool {
    let tokens = tokens;
    let mut background = false;
    if let Some(last) = tokens.last() {
        if matches!(last, Tokens::Background) {
            background = true;
            tokens.pop();
        }
    }
    if tokens.iter().any(|t| matches!(t, Tokens::Background)) {
        background = false;
    }
    if background {
        return background;
    }
    background
}

pub fn parse_pipelines(tokens: Vec<Tokens>) -> Option<Pipeline> {
    let mut tokens = tokens;
    let mut commands = Vec::new();
    let background = is_background(&mut tokens);
    while !tokens.is_empty() {
        let cmd = parse_command(&mut tokens)?;
        commands.push(cmd);
        if matches!(tokens.first(), Some(Tokens::Pipe)) {
            tokens.remove(0);
        } else {
            break;
        }
    }
    Some(Pipeline {
        commands,
        background,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Tokens;

    #[test]
    fn test_parse_simple_command() {
        let tokens = vec![
            Tokens::Word {
                value: String::from("ls"),
                quoted: false,
            },
            Tokens::Word {
                value: String::from("-l"),
                quoted: false,
            },
        ];
        let pipeline = parse_pipelines(tokens).unwrap();
        assert_eq!(pipeline.commands.len(), 1);
        assert!(pipeline.commands[0].program.ends_with("ls"));
        assert_eq!(pipeline.commands[0].args, vec!["ls", "-l"]);
    }

    #[test]
    fn test_parse_pipe() {
        let tokens = vec![
            Tokens::Word {
                value: "ls".to_string(),
                quoted: false,
            },
            Tokens::Pipe,
            Tokens::Word {
                value: "grep".to_string(),
                quoted: false,
            },
            Tokens::Word {
                value: "foo".to_string(),
                quoted: false,
            },
        ];
        let pipeline = parse_pipelines(tokens).unwrap();
        assert_eq!(pipeline.commands.len(), 2);
        assert!(pipeline.commands[0].program.ends_with("ls"));
        assert!(pipeline.commands[1].program.ends_with("grep"));
    }

    #[test]
    fn test_parse_redirection() {
        let tokens = vec![
            Tokens::Word {
                value: "echo".to_string(),
                quoted: false,
            },
            Tokens::Word {
                value: "hello".to_string(),
                quoted: false,
            },
            Tokens::Output,
            Tokens::Word {
                value: "out.txt".to_string(),
                quoted: false,
            },
        ];
        let pipeline = parse_pipelines(tokens).unwrap();
        assert_eq!(pipeline.commands.len(), 1);
        assert_eq!(pipeline.commands[0].redirects.len(), 1);
        if let crate::models::Redirects::Output(ref f) = pipeline.commands[0].redirects[0] {
            assert_eq!(f, "out.txt");
        } else {
            panic!("Expected Output");
        }
    }
    #[test]
    fn home_expansion() {
        let tokens = vec![
            Tokens::Word {
                value: String::from("cd"),
                quoted: false,
            },
            Tokens::Word {
                value: String::from("~"),
                quoted: false,
            },
        ];
        let pipeline = parse_pipelines(tokens).unwrap();
        assert_eq!(pipeline.commands.len(), 1);
        assert!(pipeline.commands[0].program.ends_with("cd"));
        assert_eq!(pipeline.commands[0].args, vec!["cd", "/home/arjun"]);
    }
}
