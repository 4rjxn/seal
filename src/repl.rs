use std::env;
use std::path::PathBuf;

use rustyline::ColorMode;
use rustyline::config::Configurer;
use rustyline::{Editor, error::ReadlineError, history::DefaultHistory};

use crate::prompt::set_prompt;

pub struct Repl {
    editor: Editor<(), DefaultHistory>,
    history_path: PathBuf,
}

impl Repl {
    pub fn new() -> Self {
        let mut editor = Editor::<(), DefaultHistory>::new().unwrap();
        //editor.set_helper(Some({}));
        editor.set_color_mode(ColorMode::Enabled);
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
                    if line.trim().is_empty() {
                        continue;
                    }
                    let _ = self.editor.add_history_entry(line.as_str());
                    let _ = self.editor.save_history(&self.history_path);
                    return Some(String::from(line));
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
