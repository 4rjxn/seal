use crate::{
    lexer::tokens::{RedirMode, Token},
    parser::{
        error::{Found, ParseError::ExpectedRedirectTarget, ParseResult},
        node::{Redirect, RedirectKind, UnitCommand},
    },
};

mod error;
mod node;

struct Parser {
    stream: Vec<Token>,
}

impl Parser {
    fn new(mut stream: Vec<Token>) -> Self {
        stream.reverse();
        Self { stream: stream }
    }
    fn bump(&mut self) -> Option<Token> {
        self.stream.pop()
    }
    fn peek(&self) -> Option<&Token> {
        self.stream.last()
    }
    fn parse_list() {}
    fn parse_and_or() {}
    fn parse_pipeline() {}
    fn parse_command() {}
    fn parse_unit_command(&mut self) -> ParseResult<UnitCommand> {
        let mut cmd = UnitCommand::default();
        loop {
            match self.peek() {
                Some(Token::Word(_)) | Some(Token::Redir { .. }) => {}
                _ => break,
            }
            match self.bump() {
                Some(Token::Word(w)) => {
                    cmd.words.push(w);
                }
                Some(Token::Redir { fd, kind }) => match self.bump() {
                    Some(Token::Word(t)) => {
                        cmd.redirects.push(Redirect {
                            fd,
                            kind: RedirectKind::from(kind),
                            target: node::RedirectTarget::File(t),
                        });
                    }
                    Some(any) => return Err(ExpectedRedirectTarget(Found::Symbol(any))),
                    None => return Err(ExpectedRedirectTarget(Found::Eof)),
                },
                _ => unreachable!(),
            }
        }
        Ok(cmd)
    }
}

impl From<RedirMode> for RedirectKind {
    fn from(value: RedirMode) -> Self {
        match value {
            RedirMode::All => RedirectKind::All,
            RedirMode::Output => RedirectKind::Output,
            RedirMode::Append => RedirectKind::Append,
            RedirMode::AllAppend => RedirectKind::AllAppend,
            RedirMode::Input => RedirectKind::Input,
        }
    }
}

#[cfg(test)]
mod parser_tester {
    use crate::parser::node::RedirectTarget;

    use super::*;

    #[test]
    fn test_single_word_command() {
        let tokens = vec![Token::Word("cat".into())];
        let unit_command = Parser::new(tokens).parse_unit_command().unwrap();
        assert_eq!(unit_command.words, vec!["cat"]);
        assert!(unit_command.redirects.is_empty());
    }

    #[test]
    fn test_multiple_words() {
        let tokens = vec![Token::Word("cat".into()), Token::Word("hello.txt".into())];
        let unit_command = Parser::new(tokens).parse_unit_command().unwrap();
        assert_eq!(unit_command.words, vec!["cat", "hello.txt"]);
        assert!(unit_command.redirects.is_empty());
    }

    #[test]
    fn test_command_with_redirection_producing_error() {
        let tokens = vec![
            Token::Word("cat".into()),
            Token::Word("hello.txt".into()),
            Token::Redir {
                fd: None,
                kind: RedirMode::Output,
            },
        ];
        let unit_command = Parser::new(tokens).parse_unit_command();
        assert!(
            matches!(unit_command, Err(ExpectedRedirectTarget(_))),
            "expected redirect target error but got {:?}",
            unit_command
        );
    }

    #[test]
    fn test_command_with_redirection() {
        let tokens = vec![
            Token::Word("cat".into()),
            Token::Word("hello.txt".into()),
            Token::Redir {
                fd: None,
                kind: RedirMode::Output,
            },
            Token::Word("h".into()),
        ];
        let unit_command = Parser::new(tokens).parse_unit_command().unwrap();
        assert_eq!(unit_command.words, vec!["cat", "hello.txt"]);
        assert_eq!(
            unit_command.redirects,
            vec![Redirect {
                fd: None,
                kind: RedirectKind::Output,
                target: RedirectTarget::File("h".into())
            }]
        );
    }
}
