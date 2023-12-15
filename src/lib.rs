use std::{path::Path, process::ExitCode, time::Duration};

mod flatzinc;
mod error;
pub mod sat;
pub mod termination;

pub fn solve_cnf(path: impl AsRef<Path>, timeout: Option<Duration>) -> ExitCode {
    match sat::run_solver(path, timeout) {
        Ok(sat::Conclusion::Satisfiable(assignment)) => {
            println!("s SATISFIABLE");
            println!("v {}", assignment.value_line());
            ExitCode::SUCCESS
        }
        Ok(sat::Conclusion::Unsatisfiable) => {
            println!("s UNSATISFIABLE");
            ExitCode::SUCCESS
        }
        Ok(sat::Conclusion::Unknown) => {
            println!("s UNKNOWN");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}

pub fn solve_flatzinc(path: impl AsRef<Path>, timeout: Option<Duration>) -> ExitCode {
    match flatzinc::run_solver(path, timeout) {
        Ok(flatzinc::Conclusion::Satisfiable(assignment)) => {
            assignment.print_output();
            println!("----------");
            ExitCode::SUCCESS
        }
        Ok(flatzinc::Conclusion::Unsatisfiable) => {
            println!("=====UNSATISFIABLE=====");
            ExitCode::SUCCESS
        }
        Ok(flatzinc::Conclusion::Unknown) => {
            println!("=====UNKNOWN=====");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}
