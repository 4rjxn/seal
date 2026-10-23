#[derive(Debug, PartialEq)]
pub enum RedirectKind {
    All,
    Output,
    Append,
    AllAppend,
    Input,
}

#[derive(Debug, PartialEq)]
pub enum RedirectTarget {
    Fd(i32),
    File(String),
}

#[derive(Debug, PartialEq)]
pub struct Redirect {
    pub fd: Option<i32>,
    pub kind: RedirectKind,
    pub target: RedirectTarget,
}

#[derive(Default, Debug, PartialEq)]
pub struct UnitCommand {
    pub words: Vec<String>,
    pub redirects: Vec<Redirect>,
}

enum Command {
    Unit(UnitCommand),
    Subshell(Box<SyntaxNode>),
}

pub enum SyntaxNode {
    And {
        left: Box<SyntaxNode>,
        right: Box<SyntaxNode>,
    },
    Or {
        left: Box<SyntaxNode>,
        right: Box<SyntaxNode>,
    },
    Background(Box<SyntaxNode>),
    Pipeline(Vec<Command>),
    Sequence(Vec<SyntaxNode>),
}
