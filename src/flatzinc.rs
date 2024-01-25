use std::{path::Path, process::ExitCode, time::Duration};

pub fn solve(_path: impl AsRef<Path>, _timeout: Option<Duration>) -> ExitCode {
    todo!()
    // Ok(flatzinc::Conclusion::Satisfiable(assignment)) => {
    //     assignment.print_output();
    //     println!("----------");
    //     ExitCode::SUCCESS
    // }
    // Ok(flatzinc::Conclusion::Unsatisfiable) => {
    //     println!("=====UNSATISFIABLE=====");
    //     ExitCode::SUCCESS
    // }
    // Ok(flatzinc::Conclusion::Unknown) => {
    //     println!("=====UNKNOWN=====");
    //     ExitCode::SUCCESS
    // }
    // Err(e) => {
    //     eprintln!("Error: {e}");
    //     ExitCode::FAILURE
    // }
}
