pub trait EscapeSpaces {
    fn escape_spaces(self) -> String;
}

impl EscapeSpaces for &str {
    fn escape_spaces(self) -> String {
        self.replace(" ", "\\ ")
    }
}
