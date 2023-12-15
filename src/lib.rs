use std::{path::Path, process::ExitCode, time::Duration};

use sat::Conclusion;

pub mod sat;
pub mod termination;

pub fn solve_cnf(path: impl AsRef<Path>, timeout: Option<Duration>) -> ExitCode {
    match sat::run_solver(path, timeout) {
        Ok(Conclusion::Satisfiable(assignment)) => {
            println!("s SATISFIABLE");
            println!("v {}", assignment.value_line());
            ExitCode::SUCCESS
        }
        Ok(Conclusion::Unsatisfiable) => {
            println!("s UNSATISFIABLE");
            ExitCode::SUCCESS
        }
        Ok(Conclusion::Unknown) => {
            println!("s UNKNOWN");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}
