use std::{
    collections::HashMap,
    ffi::CString,
    path::{Path, PathBuf},
};

pub struct Env {
    vars: HashMap<String, String>,
    cwd: PathBuf,
}

impl Env {
    pub fn new() -> Self {
        Self {
            vars: std::env::vars().collect(),
            cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
        }
    }
    pub fn get(&self, key: &str) -> Option<&str> {
        self.vars.get(key).map(|s| s.as_str())
    }
    fn set(&mut self, key: String, val: String) {
        self.vars.insert(key, val);
    }
    fn unset(&mut self, key: &str) {
        self.vars.remove(key);
    }

    pub fn as_execve_list(&self) -> Vec<CString> {
        self.vars
            .iter()
            .map(|(k, s)| CString::new(format!("{}={}", k, s)).unwrap_or_default())
            .collect()
    }
    fn cwd(&self) -> &Path {
        &self.cwd
    }
    fn set_cwd(&mut self, path: PathBuf) {
        self.cwd = path;
    }
}

impl Default for Env {
    fn default() -> Self {
        Self::new()
    }
}
