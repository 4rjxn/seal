use crate::{
    lexer::tokens::{RedirMode, Token},
    parser::{
        error::{
            Found,
            ParseError::{ExpectedCommand, ExpectedRParen, ExpectedRedirectTarget},
            ParseResult,
        },
        node::{
            Command::{self, Subshell, Unit},
            Redirect, RedirectKind, SyntaxNode, UnitCommand,
        },
    },
};

pub mod error;
pub mod node;

struct Parser {
    stream: Vec<Token>,
}

pub fn parse(stream: Vec<Token>) -> ParseResult<SyntaxNode> {
    Parser::new(stream).parse_list()
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
    fn parse_list(&mut self) -> ParseResult<SyntaxNode> {
        let mut items = Vec::new();
        loop {
            let and_or = self.parse_and_or()?;
            match self.peek() {
                Some(Token::Bg) => {
                    self.bump();
                    items.push(SyntaxNode::Background(Box::new(and_or)));
                }
                Some(Token::Semi) => {
                    self.bump();
                    items.push(and_or);
                }
                _ => {
                    items.push(and_or);
                    break;
                }
            }
            if matches!(self.peek(), None | Some(Token::RParen)) {
                break;
            }
        }
        Ok(if items.len() == 1 {
            items.pop().unwrap()
        } else {
            SyntaxNode::Sequence(items)
        })
    }
    fn parse_and_or(&mut self) -> ParseResult<SyntaxNode> {
        let mut left = self.parse_pipeline()?;
        loop {
            match self.peek() {
                Some(Token::AndIf) => {
                    self.bump();
                    let right = self.parse_pipeline()?;
                    left = SyntaxNode::And {
                        left: Box::new(left),
                        right: Box::new(right),
                    }
                }
                Some(Token::OrIf) => {
                    self.bump();
                    let right = self.parse_pipeline()?;
                    left = SyntaxNode::Or {
                        left: Box::new(left),
                        right: Box::new(right),
                    }
                }
                _ => break,
            }
        }
        Ok(left)
    }
    fn parse_pipeline(&mut self) -> ParseResult<SyntaxNode> {
        let mut cmds = vec![self.parse_command()?];
        while self.peek() == Some(&Token::Pipe) {
            self.bump();
            cmds.push(self.parse_command()?);
        }
        Ok(SyntaxNode::Pipeline(cmds))
    }
    fn parse_command(&mut self) -> ParseResult<Command> {
        if self.peek() == Some(&Token::LParen) {
            self.bump();
            let inner = self.parse_list()?;
            match self.bump() {
                Some(Token::RParen) => Ok(Subshell(Box::new(inner))),
                Some(t) => Err(ExpectedRParen(Found::Symbol(t))),
                None => Err(ExpectedRParen(Found::Eof)),
            }
        } else {
            Ok(Unit(self.parse_unit_command()?))
        }
    }
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

        if cmd.words.is_empty() && cmd.redirects.is_empty() {
            return Err(ExpectedCommand(match self.bump() {
                Some(t) => Found::Symbol(t),
                None => Found::Eof,
            }));
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
    use std::vec;

    use crate::parser::{
        error::ParseError::{ExpectedRParen, ExpectedRedirectTarget},
        node::RedirectTarget,
    };

    use super::*;

    // --- UnitCommand: words ---

    #[test]
    fn test_single_word_command() {
        let tokens = vec![Token::Word("cat".into())];
        let cmd = Parser::new(tokens).parse_unit_command().unwrap();
        assert_eq!(cmd.words, vec!["cat"]);
        assert!(cmd.redirects.is_empty());
    }

    #[test]
    fn test_multiple_words() {
        let tokens = vec![Token::Word("cat".into()), Token::Word("hello.txt".into())];
        let cmd = Parser::new(tokens).parse_unit_command().unwrap();
        assert_eq!(cmd.words, vec!["cat", "hello.txt"]);
        assert!(cmd.redirects.is_empty());
    }

    // --- UnitCommand: redirections ---

    #[test]
    fn test_output_redirection() {
        // cat hello.txt > h
        let tokens = vec![
            Token::Word("cat".into()),
            Token::Word("hello.txt".into()),
            Token::Redir {
                fd: None,
                kind: RedirMode::Output,
            },
            Token::Word("h".into()),
        ];
        let cmd = Parser::new(tokens).parse_unit_command().unwrap();
        assert_eq!(cmd.words, vec!["cat", "hello.txt"]);
        assert_eq!(
            cmd.redirects,
            vec![Redirect {
                fd: None,
                kind: RedirectKind::Output,
                target: RedirectTarget::File("h".into()),
            }]
        );
    }

    #[test]
    fn test_input_redirection() {
        // cat < input.txt
        let tokens = vec![
            Token::Word("cat".into()),
            Token::Redir {
                fd: None,
                kind: RedirMode::Input,
            },
            Token::Word("input.txt".into()),
        ];
        let cmd = Parser::new(tokens).parse_unit_command().unwrap();
        assert_eq!(cmd.words, vec!["cat"]);
        assert_eq!(cmd.redirects[0].kind, RedirectKind::Input);
    }

    #[test]
    fn test_append_redirection() {
        // echo hi >> log.txt
        let tokens = vec![
            Token::Word("echo".into()),
            Token::Word("hi".into()),
            Token::Redir {
                fd: None,
                kind: RedirMode::Append,
            },
            Token::Word("log.txt".into()),
        ];
        let cmd = Parser::new(tokens).parse_unit_command().unwrap();
        assert_eq!(cmd.words, vec!["echo", "hi"]);
        assert_eq!(cmd.redirects[0].kind, RedirectKind::Append);
    }

    #[test]
    fn test_all_redirection() {
        // cmd &> out.txt
        let tokens = vec![
            Token::Word("cmd".into()),
            Token::Redir {
                fd: None,
                kind: RedirMode::All,
            },
            Token::Word("out.txt".into()),
        ];
        let cmd = Parser::new(tokens).parse_unit_command().unwrap();
        assert_eq!(cmd.redirects[0].kind, RedirectKind::All);
    }

    #[test]
    fn test_fd_prefixed_redirection() {
        // cmd 2> err.log
        let tokens = vec![
            Token::Word("cmd".into()),
            Token::Redir {
                fd: Some(2),
                kind: RedirMode::Output,
            },
            Token::Word("err.log".into()),
        ];
        let cmd = Parser::new(tokens).parse_unit_command().unwrap();
        assert_eq!(
            cmd.redirects,
            vec![Redirect {
                fd: Some(2),
                kind: RedirectKind::Output,
                target: RedirectTarget::File("err.log".into()),
            }]
        );
    }

    #[test]
    fn test_multiple_redirections() {
        // cmd < in.txt > out.txt
        let tokens = vec![
            Token::Word("cmd".into()),
            Token::Redir {
                fd: None,
                kind: RedirMode::Input,
            },
            Token::Word("in.txt".into()),
            Token::Redir {
                fd: None,
                kind: RedirMode::Output,
            },
            Token::Word("out.txt".into()),
        ];
        let cmd = Parser::new(tokens).parse_unit_command().unwrap();
        assert_eq!(cmd.redirects.len(), 2);
        assert_eq!(cmd.redirects[0].kind, RedirectKind::Input);
        assert_eq!(cmd.redirects[1].kind, RedirectKind::Output);
    }

    // --- UnitCommand: error cases ---

    #[test]
    fn test_empty_command() {
        let result = Parser::new(vec![Token::Semi]).parse_unit_command();
        assert!(matches!(result, Err(ExpectedCommand(_))));
    }

    #[test]
    fn test_missing_redirect_target_eof() {
        // cat >   (nothing after >)
        let tokens = vec![
            Token::Word("cat".into()),
            Token::Redir {
                fd: None,
                kind: RedirMode::Output,
            },
        ];
        let result = Parser::new(tokens).parse_unit_command();
        assert!(matches!(result, Err(ExpectedRedirectTarget(_))));
    }

    #[test]
    fn test_redirect_target_is_operator_error() {
        // echo > |   (| is not a valid redirect target)
        let tokens = vec![
            Token::Word("echo".into()),
            Token::Redir {
                fd: None,
                kind: RedirMode::Output,
            },
            Token::Pipe,
        ];
        let result = parse(tokens);
        assert!(matches!(result, Err(ExpectedRedirectTarget(_))));
    }

    // --- Pipeline ---

    #[test]
    fn test_single_command_wraps_in_pipeline() {
        // echo hello → Pipeline([Unit(echo hello)])
        let tokens = vec![Token::Word("echo".into()), Token::Word("hello".into())];
        let node = parse(tokens).unwrap();
        assert!(matches!(node, SyntaxNode::Pipeline(ref cmds) if cmds.len() == 1));
    }

    #[test]
    fn test_simple_pipeline() {
        // cat file.txt | grep foo
        let tokens = vec![
            Token::Word("cat".into()),
            Token::Word("file.txt".into()),
            Token::Pipe,
            Token::Word("grep".into()),
            Token::Word("foo".into()),
        ];
        let node = parse(tokens).unwrap();
        assert!(matches!(node, SyntaxNode::Pipeline(ref cmds) if cmds.len() == 2));
    }

    #[test]
    fn test_three_stage_pipeline() {
        // cat f | grep x | wc -l
        let tokens = vec![
            Token::Word("cat".into()),
            Token::Word("f".into()),
            Token::Pipe,
            Token::Word("grep".into()),
            Token::Word("x".into()),
            Token::Pipe,
            Token::Word("wc".into()),
            Token::Word("-l".into()),
        ];
        let node = parse(tokens).unwrap();
        assert!(matches!(node, SyntaxNode::Pipeline(ref cmds) if cmds.len() == 3));
    }

    #[test]
    fn test_pipeline_with_redirect_on_last_stage() {
        // cat file.txt | grep foo > out.txt
        let tokens = vec![
            Token::Word("cat".into()),
            Token::Word("file.txt".into()),
            Token::Pipe,
            Token::Word("grep".into()),
            Token::Word("foo".into()),
            Token::Redir {
                fd: None,
                kind: RedirMode::Output,
            },
            Token::Word("out.txt".into()),
        ];
        let node = parse(tokens).unwrap();
        let SyntaxNode::Pipeline(cmds) = node else {
            panic!("expected Pipeline")
        };
        assert_eq!(cmds.len(), 2);
        let Command::Unit(last) = &cmds[1] else {
            panic!("expected Unit")
        };
        assert_eq!(last.redirects.len(), 1);
    }

    // --- Logical AND / OR ---

    #[test]
    fn test_logical_and() {
        // make && make install
        let tokens = vec![
            Token::Word("make".into()),
            Token::AndIf,
            Token::Word("make".into()),
            Token::Word("install".into()),
        ];
        let node = parse(tokens).unwrap();
        assert!(matches!(node, SyntaxNode::And { .. }));
    }

    #[test]
    fn test_logical_or() {
        // cmd || fallback
        let tokens = vec![
            Token::Word("cmd".into()),
            Token::OrIf,
            Token::Word("fallback".into()),
        ];
        let node = parse(tokens).unwrap();
        assert!(matches!(node, SyntaxNode::Or { .. }));
    }

    #[test]
    fn test_and_then_or_left_associative() {
        // a && b || c  →  Or(And(a,b), c)  (left-associative)
        let tokens = vec![
            Token::Word("a".into()),
            Token::AndIf,
            Token::Word("b".into()),
            Token::OrIf,
            Token::Word("c".into()),
        ];
        let node = parse(tokens).unwrap();
        // Outer node must be Or, left branch must be And
        let SyntaxNode::Or { left, .. } = node else {
            panic!("expected Or")
        };
        assert!(matches!(*left, SyntaxNode::And { .. }));
    }

    #[test]
    fn test_and_with_pipeline_on_left() {
        // make | tee log && make install
        let tokens = vec![
            Token::Word("make".into()),
            Token::Pipe,
            Token::Word("tee".into()),
            Token::Word("log".into()),
            Token::AndIf,
            Token::Word("make".into()),
            Token::Word("install".into()),
        ];
        let node = parse(tokens).unwrap();
        let SyntaxNode::And { left, .. } = node else {
            panic!("expected And")
        };
        // Left is a two-stage pipeline
        assert!(matches!(*left, SyntaxNode::Pipeline(ref c) if c.len() == 2));
    }

    // --- Sequence (;) ---

    #[test]
    fn test_sequence_two_commands() {
        // echo a ; echo b
        let tokens = vec![
            Token::Word("echo".into()),
            Token::Word("a".into()),
            Token::Semi,
            Token::Word("echo".into()),
            Token::Word("b".into()),
        ];
        let node = parse(tokens).unwrap();
        assert!(matches!(node, SyntaxNode::Sequence(ref items) if items.len() == 2));
    }

    #[test]
    fn test_sequence_three_commands() {
        // a ; b ; c
        let tokens = vec![
            Token::Word("a".into()),
            Token::Semi,
            Token::Word("b".into()),
            Token::Semi,
            Token::Word("c".into()),
        ];
        let node = parse(tokens).unwrap();
        assert!(matches!(node, SyntaxNode::Sequence(ref items) if items.len() == 3));
    }

    #[test]
    fn test_trailing_semicolon_ok() {
        // echo hi ;   (trailing ; with nothing after)
        let tokens = vec![
            Token::Word("echo".into()),
            Token::Word("hi".into()),
            Token::Semi,
        ];
        assert!(parse(tokens).is_ok());
    }

    // --- Background (&) ---

    #[test]
    fn test_background_command() {
        // sleep 10 &
        let tokens = vec![
            Token::Word("sleep".into()),
            Token::Word("10".into()),
            Token::Bg,
        ];
        let node = parse(tokens).unwrap();
        assert!(matches!(node, SyntaxNode::Background(_)));
    }

    #[test]
    fn test_background_then_foreground_is_sequence() {
        // sleep 10 & echo done  →  Sequence([Background(sleep), Pipeline(echo)])
        let tokens = vec![
            Token::Word("sleep".into()),
            Token::Word("10".into()),
            Token::Bg,
            Token::Word("echo".into()),
            Token::Word("done".into()),
        ];
        let node = parse(tokens).unwrap();
        let SyntaxNode::Sequence(ref items) = node else {
            panic!("expected Sequence")
        };
        assert_eq!(items.len(), 2);
        assert!(matches!(items[0], SyntaxNode::Background(_)));
        assert!(matches!(items[1], SyntaxNode::Pipeline(_)));
    }

    // --- Subshell ---

    #[test]
    fn test_simple_subshell() {
        // (echo hello)
        let tokens = vec![
            Token::LParen,
            Token::Word("echo".into()),
            Token::Word("hello".into()),
            Token::RParen,
        ];
        let node = parse(tokens).unwrap();
        assert!(matches!(
            node,
            SyntaxNode::Pipeline(ref cmds) if matches!(cmds[0], Command::Subshell(_))
        ));
    }

    #[test]
    fn test_subshell_in_pipeline() {
        // (cat file) | grep foo
        let tokens = vec![
            Token::LParen,
            Token::Word("cat".into()),
            Token::Word("file".into()),
            Token::RParen,
            Token::Pipe,
            Token::Word("grep".into()),
            Token::Word("foo".into()),
        ];
        let node = parse(tokens).unwrap();
        assert!(matches!(node, SyntaxNode::Pipeline(ref cmds) if cmds.len() == 2));
    }

    #[test]
    fn test_subshell_with_sequence_inside() {
        // (echo a ; echo b)  →  Pipeline([Subshell(Sequence([...]))])
        let tokens = vec![
            Token::LParen,
            Token::Word("echo".into()),
            Token::Word("a".into()),
            Token::Semi,
            Token::Word("echo".into()),
            Token::Word("b".into()),
            Token::RParen,
        ];
        let node = parse(tokens).unwrap();
        let SyntaxNode::Pipeline(cmds) = node else {
            panic!("expected Pipeline")
        };
        let Command::Subshell(inner) = &cmds[0] else {
            panic!("expected Subshell")
        };
        assert!(matches!(**inner, SyntaxNode::Sequence(_)));
    }

    #[test]
    fn test_nested_subshell() {
        // ((echo hi))
        let tokens = vec![
            Token::LParen,
            Token::LParen,
            Token::Word("echo".into()),
            Token::Word("hi".into()),
            Token::RParen,
            Token::RParen,
        ];
        assert!(parse(tokens).is_ok());
    }

    #[test]
    fn test_unclosed_subshell_error() {
        // (echo hello   ← missing )
        let tokens = vec![
            Token::LParen,
            Token::Word("echo".into()),
            Token::Word("hello".into()),
        ];
        let result = parse(tokens);
        assert!(matches!(result, Err(ExpectedRParen(_))));
    }
}
