#[derive(PartialEq, PartialOrd, Debug)]
pub enum RedirMode {
    All,
    Output,
    Append,
    AllAppend,
    Input,
}

#[derive(PartialEq, PartialOrd, Debug)]
pub enum Token {
    //control flow
    Pipe,
    AndIf,
    OrIf,
    Semi,
    Bg,

    // Redirections
    Redir { fd: Option<i32>, kind: RedirMode },

    // symbols
    LParen,
    RParen,

    // word
    Word(String),
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Pipe => write!(f, "|"),
            Token::AndIf => write!(f, "&&"),
            Token::OrIf => write!(f, "||"),
            Token::Semi => write!(f, ";"),
            Token::Bg => todo!(),
            Token::Redir { fd: Some(fd), kind } => write!(f, "{fd}{kind}"),
            Token::Redir { fd: None, kind } => write!(f, "{kind}"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::Word(w) => write!(f, "{w}"),
        }
    }
}

impl std::fmt::Display for RedirMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RedirMode::All => write!(f, "&>"),
            RedirMode::Output => write!(f, ">"),
            RedirMode::Append => write!(f, ">>"),
            RedirMode::AllAppend => write!(f, "&>>"),
            RedirMode::Input => write!(f, "<"),
        }
    }
}
