use std::os::fd::AsRawFd;

use nix::{
    fcntl::{OFlag, open},
    libc::dup2,
    sys::stat::Mode,
};

use crate::error::ShellResult;
use crate::models::{Command, RedirectKind};

pub fn set_redirection(command: &Command) -> ShellResult<()> {
    if command.redirects.is_empty() {
        return Ok(());
    }
    for redirect in &command.redirects {
        match redirect.kind {
            RedirectKind::Output => {
                let fd = open(
                    redirect.target.as_str(),
                    OFlag::O_CREAT | OFlag::O_WRONLY | OFlag::O_TRUNC,
                    Mode::from_bits(0o644).unwrap(),
                )?;
                unsafe { dup2(fd.as_raw_fd(), 1) };
            }
            RedirectKind::OutputErr => {
                let fd = open(
                    redirect.target.as_str(),
                    OFlag::O_CREAT | OFlag::O_WRONLY | OFlag::O_TRUNC,
                    Mode::from_bits(0o644).unwrap(),
                )?;
                unsafe { dup2(fd.as_raw_fd(), 2) };
            }
            RedirectKind::Append => {
                let fd = open(
                    redirect.target.as_str(),
                    OFlag::O_CREAT | OFlag::O_WRONLY | OFlag::O_APPEND,
                    Mode::from_bits(0o644).unwrap(),
                )?;
                unsafe { dup2(fd.as_raw_fd(), 1) };
            }
            RedirectKind::AppendErr => {
                let fd = open(
                    redirect.target.as_str(),
                    OFlag::O_CREAT | OFlag::O_WRONLY | OFlag::O_APPEND,
                    Mode::from_bits(0o644).unwrap(),
                )?;
                unsafe { dup2(fd.as_raw_fd(), 2) };
            }
        }
    }
    Ok(())
}
