use std::{cell::RefCell, rc::Rc};

use crate::models::ShellState;

pub type ShellStateType = Rc<RefCell<ShellState>>;
