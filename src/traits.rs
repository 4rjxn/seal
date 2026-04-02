use std::{
    fs::read_dir,
    io::{self, Error},
};

pub trait Globbing {
    fn expand_glob(&self) -> Result<Vec<String>, Error>;
}

impl Globbing for String {
    fn expand_glob(&self) -> Result<Vec<String>, Error> {
        let mut dir = String::new();
        for char in self.chars() {
            match char {
                '*' => {
                    let items = read_dir(dir)?
                        .map(|res| Ok(res?.path().to_string_lossy().into_owned()))
                        .collect::<Result<Vec<_>, io::Error>>()?;
                    return Ok(items);
                }
                _ => {
                    dir.push(char);
                }
            }
        }
        Err(Error::new(io::ErrorKind::NotFound, "parse err!!"))
    }
}

pub trait EscapeTrait {
    fn escape_spaces(&self) -> String;
}

impl EscapeTrait for str {
    fn escape_spaces(&self) -> String {
        self.replace(" ", "\\ ")
    }
}
