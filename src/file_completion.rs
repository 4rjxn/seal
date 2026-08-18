use std::{
    borrow::Cow,
    fs, io,
    path::{Path, PathBuf},
};

use rustyline::{
    Helper,
    completion::{Completer, Pair},
    error::ReadlineError,
    highlight::{CmdKind, Highlighter},
    hint::Hinter,
    validate::Validator,
};

use crate::{
    traits::EscapeTrait,
    utils::{get_path_from_env, is_builtin},
};

pub struct FileCompletion {}

impl Completer for FileCompletion {
    type Candidate = Pair;
    fn complete(
        &self,
        line: &str,
        _pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        let total_length = line.len();
        let prefix;
        match line.split_whitespace().last() {
            Some(l) => prefix = l,
            None => prefix = "",
        }
        let child = match prefix.split("/").last() {
            Some(last) => last,
            None => "",
        };
        let parent = match prefix.strip_suffix(child) {
            Some(p) => p,
            None => prefix,
        };
        let final_length = total_length - prefix.len();
        let mut matches = Vec::new();
        let path = Path::new(".").join(&parent);
        let mut entries = match fs::read_dir(&path) {
            Ok(path) => path
                .map(|res| res.map(|e| e.path()))
                .collect::<Result<Vec<_>, io::Error>>()?,
            Err(_) => {
                vec![]
            }
        };
        entries.sort_by_key(|s| s.to_str().unwrap().len());
        for entry in entries {
            let file_name = entry.file_name();
            let name = file_name.unwrap().to_str().unwrap();
            if prefix == "" {
                let name = name.escape_spaces();
                matches.push(Pair {
                    display: name.to_string(),
                    replacement: name.to_string(),
                });
            } else if name.to_lowercase().starts_with(&child.to_lowercase()) {
                let name = name.escape_spaces();
                matches.push(Pair {
                    display: name.to_string(),
                    replacement: produce_replacement(&path, name, parent.to_string()),
                });
            }
        }
        Ok((final_length, matches))
    }
}

fn produce_replacement(path: &PathBuf, name: String, parent: String) -> String {
    if path.join(&name).is_file() {
        format!("{}{}", parent, name)
    } else {
        format!("{}{}/", parent, name)
    }
}

impl Hinter for FileCompletion {
    type Hint = String;
}

impl Highlighter for FileCompletion {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        Cow::Owned(highlight_words(line))
    }
    fn highlight_char(&self, _line: &str, _pos: usize, kind: CmdKind) -> bool {
        if kind == CmdKind::Other {
            return true;
        }
        return false;
    }
}

fn highlight_words<'l>(line: &'l str) -> String {
    let words = line.split_whitespace();
    let mut highlighted_sting = String::new();
    words.for_each(|word| match get_path_from_env(&word.to_string()) {
        Ok(_) => highlighted_sting.push_str(&format!("\x1b[1;97m{}\x1b[0m ", word)),
        Err(_) => match is_builtin(word) {
            Some(_) => highlighted_sting.push_str(&format!("\x1b[1;93m{}\x1b[0m ", word)),
            None => highlighted_sting.push_str(&format!("{} ", word)),
        },
    });
    highlighted_sting
}

impl Validator for FileCompletion {}
impl Helper for FileCompletion {}
