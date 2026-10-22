use thiserror::Error;

#[derive(Error, Debug)]
pub enum LexerError {
    #[error("Closing single quote is missing.")]
    UnterminatedSingleQuote,
    #[error("Closing double quote is missing.")]
    UnterminatedDoubleQuote,
    #[error("Trailing Backslash found.")]
    TrailingBackslash,
}
