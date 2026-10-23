use thiserror::Error;

use crate::lexer::tokens::Token;

#[derive(Error, Debug, PartialEq)]
pub enum ParseError {
    #[error("expected a command, found {0}")]
    ExpectedCommand(Found),

    #[error("expected a file name after redirection, found {0}")]
    ExpectedRedirectTarget(Found),

    #[error("expected ')', found {0}")]
    ExpectedRParen(Found),

    #[error("unexpected token {0}")]
    UnexpectedToken(Found),
}

#[derive(Debug, PartialEq)]
pub enum Found {
    Eof,
    Symbol(Token),
}

impl std::fmt::Display for Found {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Found::Eof => write!(f, "EOF"),
            Found::Symbol(s) => write!(f, "'{s}'"),
        }
    }
}

//custom Result type of parser
pub type ParseResult<T> = Result<T, ParseError>;
