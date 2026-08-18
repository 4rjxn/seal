use core::str;

use crate::models::{LexerState, Token};

pub fn tokenize(input: &str) -> Vec<Token> {
    Lexer::new(input).tokenize()
}

struct Lexer<'a> {
    input: str::Chars<'a>,
    state: LexerState,
    current_word: String,
    was_quoted: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input: input.trim().chars(),
            state: LexerState::Normal,
            current_word: String::new(),
            was_quoted: false,
        }
    }
    pub fn tokenize(mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while let Some(ch) = self.input.next() {
            if let Some(token) = self.step(ch) {
                tokens.push(token);
            }
        }
        self.flush_word(&mut tokens);
        tokens
    }
}

impl<'a> Lexer<'a> {
    fn step(&mut self, ch: char) -> Option<Token> {
        match &self.state.clone() {
            LexerState::Escaped { return_to } => {
                self.current_word.push(ch);
                self.state = *return_to.clone();
                None
            }
            LexerState::InSingleQuote => self.step_in_single_quote(ch),
            LexerState::InDoubleQuote => self.step_in_double_quote(ch),
            LexerState::Normal => self.step_normal(ch),
            LexerState::LuaString => self.step_lua_string(ch),
        }
    }
    fn step_lua_string(&mut self, ch: char) -> Option<Token> {
        self.current_word.push(ch);
        None
    }
    fn step_normal(&mut self, ch: char) -> Option<Token> {
        match ch {
            ',' => {
                self.state = LexerState::LuaString;
                self.escape_for_lua()
            }
            '\\' => {
                self.state = LexerState::Escaped {
                    return_to: Box::new(LexerState::Normal),
                };
                None
            }
            '\'' => {
                self.enter_quote(LexerState::InSingleQuote);
                None
            }
            '"' => {
                self.enter_quote(LexerState::InDoubleQuote);
                None
            }
            ' ' | '\t' => self.flush_current_word(),
            _ => {
                self.current_word.push(ch);
                None
            }
        }
    }

    fn step_in_single_quote(&mut self, ch: char) -> Option<Token> {
        match ch {
            '\'' => {
                self.state = LexerState::Normal;
                None
            }
            _ => {
                self.current_word.push(ch);
                None
            }
        }
    }

    fn step_in_double_quote(&mut self, ch: char) -> Option<Token> {
        match ch {
            '"' => {
                self.state = LexerState::Normal;
                None
            }
            '\\' => {
                self.state = LexerState::Escaped {
                    return_to: Box::new(LexerState::InDoubleQuote),
                };
                None
            }
            _ => {
                self.current_word.push(ch);
                None
            }
        }
    }
    fn escape_for_lua(&mut self) -> Option<Token> {
        self.current_word = String::from("slua");
        self.flush_current_word()
    }
}

//some helper methodes;
impl<'a> Lexer<'a> {
    fn enter_quote(&mut self, state: LexerState) {
        self.state = state;
        self.was_quoted = true;
    }

    fn flush_current_word(&mut self) -> Option<Token> {
        if self.current_word.is_empty() {
            return None;
        }
        let token = word_to_token(&self.current_word, self.was_quoted);
        self.current_word.clear();
        Some(token)
    }
    fn flush_word(&mut self, tokens: &mut Vec<Token>) {
        if let Some(t) = self.flush_current_word() {
            tokens.push(t);
        }
    }
}
fn word_to_token(word: &str, quoted: bool) -> Token {
    match word {
        "|" => Token::Pipe,
        "&" => Token::Background,
        ">" | "1>" => Token::Output,
        ">>" => Token::Append,
        "2>" => Token::OutputErr,
        "2>>" => Token::AppendErr,
        _ => Token::Word {
            value: word.to_owned(),
            quoted,
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::{lexer::tokenize, models::Token};

    #[test]
    fn check_basic_tokenization() {
        let input = "ls -lah | grep a";
        let tokens = tokenize(input);
        assert_eq!(
            tokens,
            vec![
                Token::Word {
                    value: "ls".to_string(),
                    quoted: false
                },
                Token::Word {
                    value: "-lah".to_string(),
                    quoted: false
                },
                Token::Pipe,
                Token::Word {
                    value: "grep".to_string(),
                    quoted: false
                },
                Token::Word {
                    value: "a".to_string(),
                    quoted: false
                }
            ]
        )
    }
}
