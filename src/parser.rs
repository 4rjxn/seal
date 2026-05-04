use crate::{
    models::{Command, Pipeline, Redirect, RedirectKind, Token},
    traits::Expantions,
};

pub fn parse_tokens(tokens: Vec<Token>) -> Pipeline {
    let mut parser = Parser::new(tokens);
    parser.parse()
}

// The parser struct which holds parsing state;
struct Parser {
    stream: TokenStream,
    is_background: bool,
}

impl Parser {
    pub fn new(input: Vec<Token>) -> Self {
        let stream = TokenStream::new(input);
        Self {
            stream,
            is_background: false,
        }
    }

    pub fn parse(&mut self) -> Pipeline {
        let commands = self.parse_pipeline();
        Pipeline {
            commands,
            background: self.is_background,
        }
    }
}

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
impl Parser {
    fn parse_pipeline(&mut self) -> Vec<Command> {
        let mut commands = Vec::new();
        loop {
            if let Some(cmd) = self.parse_command() {
                commands.push(cmd);
            }

            match self.stream.peek() {
                Some(Token::Pipe) => self.stream.next(),
                _ => break,
            };
        }
        commands
    }

    fn parse_command(&mut self) -> Option<Command> {
        let (program, first_arg) = self.parse_program()?;
        let (args, redirects) = self.parse_args_and_redirects(first_arg);
        Some(Command {
            program,
            args,
            redirects,
        })
    }

    fn parse_args_and_redirects(&mut self, first_arg: String) -> (Vec<String>, Vec<Redirect>) {
        let mut args = vec![first_arg];
        let mut redirects = Vec::new();

        while let Some(token) = self.stream.peek() {
            match token {
                Token::Word { .. } => self.parse_arg(&mut args),
                Token::Output | Token::OutputErr | Token::Append | Token::AppendErr => {
                    self.parse_redirect(&mut redirects)
                }
                Token::Pipe | Token::Background => break,
            }
        }
        (args, redirects)
    }

    fn parse_arg(&mut self, args: &mut Vec<String>) {
        if let Some(Token::Word { value, quoted }) = self.stream.next() {
            if *quoted {
                args.push(value.clone());
            } else {
                let expanded = value.expand_path().expand_glob();
                args.extend(expanded);
            }
        }
    }

    fn parse_redirect(&mut self, redirects: &mut Vec<Redirect>) {
        let kind = match self.stream.next() {
            Some(Token::Output) => RedirectKind::Output,
            Some(Token::OutputErr) => RedirectKind::OutputErr,
            Some(Token::Append) => RedirectKind::Append,
            Some(Token::AppendErr) => RedirectKind::AppendErr,
            _ => return,
        };
        if let Some(target) = self.stream.expect_word() {
            redirects.push(Redirect { kind, target });
        } else {
            eprintln!("syntax error: expected filename after redirect");
        }
    }

    fn parse_program(&mut self) -> Option<(String, String)> {
        let word = self.stream.expect_word()?;
        let path = word.clone();
        Some((path, word))
    }
}

#[cfg(test)]
mod test {}
