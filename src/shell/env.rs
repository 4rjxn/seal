use std::{
    collections::HashMap,
    ffi::{CString, OsString},
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};

pub struct Env {
    vars: HashMap<String, OsString>,
    cwd: PathBuf,
}

impl Env {
    pub fn new() -> Self {
        Self {
            vars: std::env::vars_os()
                .filter_map(|(k, v)| Some((k.into_string().ok()?, v)))
                .collect(),
            cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
        }
    }
    pub fn get(&self, key: &str) -> Option<&str> {
        self.vars.get(key).and_then(|f| f.to_str())
    }
    pub fn set(&mut self, key: impl Into<String>, val: impl Into<OsString>) {
        self.vars.insert(key.into(), val.into());
    }
    fn _unset(&mut self, key: &str) {
        self.vars.remove(key);
    }

    pub fn as_execve_list(&self) -> Vec<CString> {
        self.vars
            .iter()
            .filter_map(|(k, v)| {
                let mut bytes = k.as_bytes().to_vec();
                bytes.push(b'=');
                bytes.extend_from_slice(v.as_bytes());
                CString::new(bytes).ok()
            })
            .collect()
    }
    pub fn cwd(&self) -> &Path {
        &self.cwd
    }
    pub fn set_cwd(&mut self, path: PathBuf) {
        self.cwd = path;
    }
}

impl Default for Env {
    fn default() -> Self {
        Self::new()
    }
}
