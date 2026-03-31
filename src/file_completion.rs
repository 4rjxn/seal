use rustyline::{
    Helper,
    completion::{Completer, FilenameCompleter, Pair},
    error::ReadlineError,
    highlight::Highlighter,
    hint::Hinter,
    validate::Validator,
};

pub struct FileCompletion {
    pub completer: FilenameCompleter,
}

impl Completer for FileCompletion {
    type Candidate = Pair;
    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &rustyline::Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        self.completer.complete(line, pos, ctx)
    }
}

impl Hinter for FileCompletion {
    type Hint = String;
}

impl Highlighter for FileCompletion {}
impl Validator for FileCompletion {}
impl Helper for FileCompletion {}
