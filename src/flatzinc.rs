use std::{path::Path, time::Duration};

use crate::error::LimigaError;

pub enum Conclusion {
    Satisfiable(Assignment),
    Unsatisfiable,
    Unknown,
}

pub fn run_solver(
    path: impl AsRef<Path>,
    timeout: Option<Duration>,
) -> Result<Conclusion, LimigaError> {
    todo!()
}

pub struct Assignment;

impl Assignment {
    pub fn print_output(&self) {
        todo!()
    }
}
