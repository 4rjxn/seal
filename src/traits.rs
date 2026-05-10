use std::{env, fs::read_dir, path::PathBuf};

use crate::types::ShellStateType;

pub trait Expantions {
    fn expand_glob(&self) -> Vec<String>;
    fn expand_path(&self) -> String;
    fn expand_env(&self, state: ShellStateType) -> String;
}

fn match_pattern(data: &str, pat: &str) -> bool {
    let data_chars: Vec<_> = data.chars().collect();
    let pattern_chars: Vec<_> = pat.chars().collect();
    let (mut i, mut j) = (0, 0);
    let mut star_idx = None;
    let mut match_idx = 0;
    while i < data_chars.len() {
        if j < pattern_chars.len() && data_chars[i] == pattern_chars[j] {
            i += 1;
            j += 1;
        } else if j < pattern_chars.len() && pattern_chars[j] == '*' {
            star_idx = Some(j);
            match_idx = i;
            j += 1;
        } else if let Some(star_pos) = star_idx {
            j = star_pos + 1;
            match_idx += 1;
            i = match_idx;
        } else {
            return false;
        }
    }
    while j < pattern_chars.len() && pattern_chars[j] == '*' {
        j += 1;
    }
    j == pattern_chars.len()
}

impl Expantions for String {
    fn expand_glob(&self) -> Vec<String> {
        if !self.contains("*") {
            return vec![self.to_owned()];
        }
        let path = PathBuf::from(self);
        let mut namepat = path.file_name().unwrap().to_str().unwrap();
        let dir = match self.chars().next() {
            Some('/') => self.replacen(namepat, "", 1),
            _ => {
                let pat = self.strip_prefix("./").unwrap_or(self);
                namepat = pat;
                String::from("./")
            }
        };
        let mut result = Vec::new();
        let entries = match read_dir(dir) {
            Ok(it) => it
                .filter_map(|res| res.ok())
                .filter(|f| {
                    f.file_name()
                        .to_str()
                        .map(|st| !st.starts_with('.'))
                        .unwrap_or(false)
                })
                .map(|e| e.path())
                .collect::<Vec<_>>(),
            Err(_) => {
                return result;
            }
        };
        for ent in entries {
            if match_pattern(ent.file_name().unwrap().to_str().unwrap(), namepat) {
                result.push(ent.to_str().unwrap().to_string());
            }
        }
        result
    }

    fn expand_path(&self) -> String {
        let home = env::var("HOME").unwrap_or_else(|_| "/".to_string());
        self.replacen("~", &home, 1)
    }

    fn expand_env(&self, state: ShellStateType) -> String {
        if self.starts_with("$") {
            let k = self.strip_prefix("$").unwrap();
            let val = {
                let s = state.borrow();
                s.get_env(&k.to_string())
            };
            match val {
                Some(v) => {
                    return v;
                }
                None => {}
            };
        }
        self.clone()
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

#[cfg(test)]
mod tests {
    use crate::traits::match_pattern;

    #[test]
    fn match_pattern_test_empty() {
        let match_stat = match_pattern("", "");
        assert_eq!(match_stat, true);
    }
    #[test]
    fn match_end_star() {
        let match_stat = match_pattern("data.txt", "data.*");
        assert_eq!(match_stat, true);
    }
    #[test]
    fn match_start_dot() {
        let match_stat = match_pattern(".data.txt", ".*");
        assert_eq!(match_stat, true);
    }
    #[test]
    fn match_start_star() {
        let match_stat = match_pattern("Data.txt", "D*");
        assert_eq!(match_stat, true);
        let match_stat = match_pattern("data.js", "*.txt");
        assert_eq!(match_stat, false);
    }
}
