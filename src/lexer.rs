use crate::models::Tokens;

pub fn tokenize(input: &str) -> Vec<Tokens> {
    let mut tokens = Vec::new();
    let mut word = String::new();
    let mut is_quote = false;
    let mut is_double_quote = false;
    let mut is_black_slash = false;

    if input.is_empty() {
        return tokens;
    }

    for char in input.trim().chars() {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_command() {
        let input = "ls -l";
        let tokens = tokenize(input);
        assert_eq!(tokens.len(), 2);
        if let Tokens::Word(ref w) = tokens[0] {
            assert_eq!(w, "ls");
        } else {
            panic!("Expected Word");
        }
    }

    #[test]
    fn test_pipes() {
        let input = "cat file | grep pattern";
        let tokens = tokenize(input);
        assert_eq!(tokens.len(), 5);
        assert!(matches!(tokens[2], Tokens::Pipe));
    }

    #[test]
    fn test_redirection() {
        let input = "echo hello > out.txt 2> err.txt";
        let tokens = tokenize(input);
        assert_eq!(tokens.len(), 6);
        assert!(matches!(tokens[2], Tokens::Output));
        assert!(matches!(tokens[4], Tokens::OutputErr));
    }

    #[test]
    fn test_quotes() {
        let input = "echo 'hello world' \"foo bar\"";
        let tokens = tokenize(input);
        assert_eq!(tokens.len(), 3);
        if let Tokens::Word(ref w) = tokens[1] {
            assert_eq!(w, "hello world");
        } else {
            panic!("Expected Word");
        }
        if let Tokens::Word(ref w) = tokens[2] {
            assert_eq!(w, "foo bar");
        } else {
            panic!("Expected Word");
        }
    }
}
