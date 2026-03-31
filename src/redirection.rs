use std::os::fd::AsRawFd;

use nix::{
    fcntl::{OFlag, open},
    libc::dup2,
    sys::stat::Mode,
};

use crate::models::{Command, Redirects};

pub fn set_redirection(command: &Command) {
    if command.redirects.is_empty() {
        return;
    }
    for redirect in &command.redirects {
        match redirect {
            Redirects::Output(file) => {
                let fd = open(
                    file.as_str(),
                    OFlag::O_CREAT | OFlag::O_WRONLY | OFlag::O_TRUNC,
                    Mode::from_bits(0o644).unwrap(),
                )
                .expect("open failed");
                unsafe { dup2(fd.as_raw_fd(), 1) };
            }
            Redirects::OutputErr(file) => {
                let fd = open(
                    file.as_str(),
                    OFlag::O_CREAT | OFlag::O_WRONLY | OFlag::O_TRUNC,
                    Mode::from_bits(0o644).unwrap(),
                )
                .expect("open failed");
                unsafe { dup2(fd.as_raw_fd(), 2) };
            }
            Redirects::Append(file) => {
                let fd = open(
                    file.as_str(),
                    OFlag::O_CREAT | OFlag::O_WRONLY | OFlag::O_APPEND,
                    Mode::from_bits(0o644).unwrap(),
                )
                .expect("open failed");
                unsafe { dup2(fd.as_raw_fd(), 1) };
            }
            Redirects::AppendErr(file) => {
                let fd = open(
                    file.as_str(),
                    OFlag::O_CREAT | OFlag::O_WRONLY | OFlag::O_APPEND,
                    Mode::from_bits(0o644).unwrap(),
                )
                .expect("open failed");
                unsafe { dup2(fd.as_raw_fd(), 2) };
            }
        }
    }
}
