use std::io;

use nix::unistd::Pid;

use crate::{
    parser::node::{Command, Redirect, SyntaxNode},
    shell::{self, Shell},
};

fn execute(shell: &mut Shell, syntax: SyntaxNode) -> i32 {
    todo!()
}

fn execute_and_or(shell: &mut Shell, left: SyntaxNode, right: SyntaxNode, is_and: bool) -> i32 {
    todo!()
}

fn execute_sequence(shell: &mut Shell, nodes: Vec<SyntaxNode>) -> i32 {
    todo!()
}

fn execute_pipeline(shell: &mut Shell, cmds: Vec<Command>, is_background: bool) -> i32 {
    todo!()
}

fn child_exec(shell: &mut Shell, cmd: Command) {}

fn apply_redirects(redirects: Vec<Redirect>) -> io::Result<()> {
    todo!()
}

fn wait_pipeline(shell: &mut Shell, pgid: Pid, children: Vec<Pid>, is_background: bool) -> i32 {
    todo!()
}
