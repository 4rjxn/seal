use std::{
    iter::Peekable,
    vec::{self, IntoIter},
};

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
