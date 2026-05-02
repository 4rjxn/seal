use nix::unistd::Pid;

pub struct Pipeline {
    pub commands: Vec<Command>,
    pub background: bool,
}

pub enum Builtins {
    Echo,
    Exit,
    Type,
    Pwd,
    Cd,
    Fg,
    Jobs,
}

pub struct ShellState {
    pub recent_id: usize,
    pub jobs: Vec<Job>,
}

#[derive(Clone)]
pub enum JobStatus {
    Running,
    Suspended,
    Done,
}

#[derive(Clone)]
pub struct Job {
    pub id: usize,
    pub pgid: Pid,
    pub status: JobStatus,
    pub command: Command,
    pub childrens: Vec<Pid>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LexerState {
    Normal,
    InSingleQuote,
    InDoubleQuote,
    Escaped { return_to: Box<LexerState> },
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Word { value: String, quoted: bool },
    Pipe,
    Output,
    Append,
    OutputErr,
    AppendErr,
    Background,
}

#[derive(Debug, Clone)]
pub struct Redirect {
    pub kind: RedirectKind,
    pub target: String,
}

#[derive(Debug, Clone)]
pub enum RedirectKind {
    Output,
    OutputErr,
    Append,
    AppendErr,
}

#[derive(Debug, Clone)]
pub struct Command {
    pub program: String,
    pub args: Vec<String>,
    pub redirects: Vec<Redirect>,
}
