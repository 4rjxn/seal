use std::env;
use std::path::PathBuf;

use rustyline::Cmd::{HistorySearchBackward, HistorySearchForward};
use rustyline::config::Configurer;
use rustyline::{ColorMode, KeyCode, KeyEvent, Modifiers};
use rustyline::{Editor, error::ReadlineError, history::DefaultHistory};

use crate::prompt::set_prompt;
use crate::repl::file_completion::FileCompletion;

mod file_completion;
mod string_traits;

pub struct Repl {
    editor: Editor<FileCompletion, DefaultHistory>,
    history_path: PathBuf,
}

impl Repl {
    pub fn new() -> Self {
        let mut editor = Editor::<FileCompletion, DefaultHistory>::new().unwrap();
        editor.set_helper(Some(FileCompletion {}));
        editor.set_color_mode(ColorMode::Enabled);
        editor.bind_sequence(
            KeyEvent(KeyCode::Up, Modifiers::NONE),
            HistorySearchBackward,
        );
        editor.bind_sequence(
            KeyEvent(KeyCode::Down, Modifiers::NONE),
            HistorySearchForward,
        );
        editor.bind_sequence(KeyEvent::ctrl('p'), HistorySearchBackward);
        editor.bind_sequence(KeyEvent::ctrl('n'), HistorySearchForward);
        let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let history_path = PathBuf::from(home).join(".seal_history");

        if history_path.exists() {
            let _ = editor.load_history(&history_path);
        }

        Self {
            editor,
            history_path,
        }
    }

    pub fn read(&mut self) -> Option<String> {
        loop {
            let readline = self.editor.readline(set_prompt().as_str());
            match readline {
                Ok(line) => {
                    let _ = self.editor.add_history_entry(line.as_str());
                    let _ = self.editor.save_history(&self.history_path);
                    return Some(line);
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

impl Default for Repl {
    fn default() -> Self {
        Self::new()
    }
}
