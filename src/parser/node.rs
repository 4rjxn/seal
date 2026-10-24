#[derive(Debug, Clone, PartialEq)]
pub enum RedirectKind {
    All,
    Output,
    Append,
    AllAppend,
    Input,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RedirectTarget {
    Fd(i32),
    File(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Redirect {
    pub fd: Option<i32>,
    pub kind: RedirectKind,
    pub target: RedirectTarget,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct UnitCommand {
    pub words: Vec<String>,
    pub redirects: Vec<Redirect>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Unit(UnitCommand),
    Subshell(Box<SyntaxNode>),
}

#[derive(Debug, Clone, PartialEq)]
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
