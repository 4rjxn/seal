mod error;
mod tokens;

use crate::lexer::{
    error::LexerError::{self, UnterminatedDoubleQuote},
    tokens::{RedirMode, Token},
};

struct Lexer<'a> {
    chars: &'a str,
    pos: usize,
}

impl<'a> Lexer<'a> {
    fn new(str: &'a str) -> Self {
        Self { chars: str, pos: 0 }
    }
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.next();
            } else {
                break;
            }
        }
    }
    fn next(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        Some(c)
    }
    fn eat(&mut self, c: char) -> bool {
        if self.peek() == Some(c) {
            self.pos += c.len_utf8();
            true
        } else {
            false
        }
    }
    fn peek(&self) -> Option<char> {
        self.chars[self.pos..].chars().next()
    }

    fn lex_redirection(&mut self, fd: Option<i32>) -> Token {
        let kind = if self.next() == Some('>') {
            if self.eat('>') {
                if self.eat('>') {
                    RedirMode::AllAppend
                } else {
                    RedirMode::Append
                }
            } else {
                RedirMode::Output
            }
        } else {
            RedirMode::Input
        };
        Token::Redir { fd, kind }
    }
    fn fd_prefix(&mut self) -> Option<i32> {
        let rest = &self.chars[self.pos..];
        let n = rest.bytes().take_while(u8::is_ascii_digit).count();
        match rest.as_bytes().get(n) {
            Some(b'>') | Some(b'<') => {
                let fd = rest[..n].parse().ok()?;
                self.pos += n;
                Some(fd)
            }
            _ => None,
        }
    }
    fn read_word(&mut self) -> Result<String, LexerError> {
        let mut word_buff = String::new();
        while let Some(c) = self.peek() {
            if is_word_end(c) {
                break;
            }
            self.next();
            match c {
                '\'' => loop {
                    match self.next() {
                        Some('\'') => break,
                        Some(c) => word_buff.push(c),
                        None => return Err(LexerError::UnterminatedSingleQuote),
                    }
                },
                '"' => self.read_double_quoted(&mut word_buff)?,
                '\\' => match self.next() {
                    Some(c) => word_buff.push(c),
                    None => return Err(LexerError::TrailingBackslash),
                },
                _ => word_buff.push(c),
            }
        }
        Ok(word_buff)
    }
    fn read_double_quoted(&mut self, word_buff: &mut String) -> Result<(), LexerError> {
        loop {
            match self.next() {
                None => return Err(UnterminatedDoubleQuote),
                Some('"') => return Ok(()),
                Some('\\') => match self.next() {
                    Some(c @ ('"' | '$' | '\\')) => word_buff.push(c),
                    Some('\n') => {}
                    Some(c) => {
                        word_buff.push('\\');
                        word_buff.push(c);
                    }
                    None => return Err(UnterminatedDoubleQuote),
                },
                Some(c) => word_buff.push(c),
            }
        }
    }
    fn lex_token(&mut self) -> Result<Option<Token>, LexerError> {
        let Some(c) = self.peek() else {
            return Ok(None);
        };

        let tok = match c {
            '|' => {
                self.next();
                if self.eat('|') {
                    Token::OrIf
                } else {
                    Token::Pipe
                }
            }
            '&' => {
                self.next();
                if self.eat('&') {
                    Token::AndIf
                } else if self.eat('>') {
                    Token::Redir {
                        fd: None,
                        kind: RedirMode::All,
                    }
                } else {
                    Token::Bg
                }
            }
            ';' => {
                self.next();
                Token::Semi
            }
            ')' => {
                self.next();
                Token::RParen
            }
            '(' => {
                self.next();
                Token::LParen
            }
            '>' | '<' => self.lex_redirection(None),
            c if c.is_ascii_digit() => match self.fd_prefix() {
                Some(fd) => self.lex_redirection(Some(fd)),
                None => Token::Word(self.read_word()?),
            },
            _ => Token::Word(self.read_word()?),
        };
        Ok(Some(tok))
    }
}
fn is_word_end(c: char) -> bool {
    c.is_whitespace() || matches!(c, '|' | '&' | ';' | '(' | ')' | '<' | '>')
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token, LexerError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();
        match self.lex_token() {
            Ok(Some(tok)) => Some(Ok(tok)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

#[cfg(test)]

mod lexer_tester {

    use super::*;

    #[test]
    fn test_a_normal_command() {
        let command = String::from("echo helloo");
        let lexer = Lexer::new(&command);
        let output: Result<Vec<Token>, LexerError> = lexer.collect();
        let tokens = match output {
            Ok(t) => t,
            Err(e) => panic!("{}", e),
        };
        assert_eq!(
            vec![
                Token::Word(String::from("echo")),
                Token::Word(String::from("helloo"))
            ],
            tokens
        );
    }

    #[test]
    fn test_a_redirection() {
        let command = String::from("echo helloo>h.txt");
        let lexer = Lexer::new(&command);
        let output: Result<Vec<Token>, LexerError> = lexer.collect();
        let tokens = match output {
            Ok(t) => t,
            Err(e) => panic!("{}", e),
        };
        assert_eq!(
            vec![
                Token::Word(String::from("echo")),
                Token::Word(String::from("helloo")),
                Token::Redir {
                    fd: None,
                    kind: RedirMode::Output
                },
                Token::Word(String::from("h.txt"))
            ],
            tokens
        );
    }
}
