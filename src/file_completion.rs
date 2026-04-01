use std::{fs, io, path::Path};

use rustyline::{
    Helper,
    completion::{Completer, Pair},
    error::ReadlineError,
    highlight::Highlighter,
    hint::Hinter,
    validate::Validator,
};

pub struct FileCompletion {}

impl Completer for FileCompletion {
    type Candidate = Pair;
    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        let prefix = &line[..pos].to_lowercase();
        let total_length = prefix.len();
        let prefix = prefix.split_whitespace().last().unwrap();
        let final_length = total_length - prefix.len();
        let mut matches = Vec::new();
        let path = Path::new(".");
        let mut entries = fs::read_dir(path)?
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, io::Error>>()?;
        entries.sort_by_key(|s| s.to_str().unwrap().len());
        for entry in entries {
            let file_name = entry.file_name();
            let name = file_name.unwrap().to_str().unwrap();

            if name.to_lowercase().starts_with(prefix) {
                matches.push(Pair {
                    display: name.to_string(),
                    replacement: name.to_string(),
                });
            }
        }

        Ok((final_length, matches))
    }
}

impl Hinter for FileCompletion {
    type Hint = String;
}

impl Highlighter for FileCompletion {}
impl Validator for FileCompletion {}
impl Helper for FileCompletion {}
