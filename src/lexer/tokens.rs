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
