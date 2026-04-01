use std::env;
use std::path::PathBuf;

use rustyline::completion::FilenameCompleter;
use rustyline::{Editor, error::ReadlineError, history::DefaultHistory};

use crate::error::ShellResult;
use crate::file_completion::FileCompletion;
use crate::lexer::tokenize;
use crate::models::Tokens;
use crate::prompt::set_prompt;

pub struct Repl {
    editor: Editor<FileCompletion, DefaultHistory>,
    history_path: PathBuf,
}

impl Repl {
    pub fn new() -> ShellResult<Self> {
        let mut editor = Editor::<FileCompletion, DefaultHistory>::new()
            .map_err(|e| crate::error::ShellError::ReadlineError(e.to_string()))?;
        editor.set_helper(Some(FileCompletion {
            completer: FilenameCompleter::new(),
        }));
        let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let history_path = PathBuf::from(home).join(".seal_history");

        if history_path.exists() {
            let _ = editor.load_history(&history_path);
        }

        Ok(Self {
            editor,
            history_path,
        })
    }

    pub fn read_and_parse(&mut self) -> Option<Vec<Tokens>> {
        loop {
            let readline = self.editor.readline(set_prompt().as_str());
            match readline {
                Ok(line) => {
                    if line.trim().is_empty() {
                        continue;
                    }
                    let _ = self.editor.add_history_entry(line.as_str());
                    let _ = self.editor.save_history(&self.history_path);
                    return Some(tokenize(&line));
                }
                Err(ReadlineError::Interrupted) => {
                    continue;
                }
                Err(ReadlineError::Eof) => {
                    return None;
                }
                Err(err) => {
                    eprintln!("Readline error: {:?}", err);
                    continue;
                }
            }
        }
    }
}
