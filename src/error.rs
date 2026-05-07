use thiserror::Error;

#[derive(Error, Debug)]
pub enum ShellError {
    #[error("Lua engine Error.")]
    LuaError,
    #[error("command not found: {0}")]
    CommandNotFound(String),
    //#[error("invalid builtin: {0}")]
    //InvalidBuiltin(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("system error: {0}")]
    Nix(#[from] nix::Error),
    #[error("parse error: {0}")]
    ReadlineError(String),
}

pub type ShellResult<T> = Result<T, ShellError>;
