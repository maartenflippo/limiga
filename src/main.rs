use std::{io::Write, path::PathBuf};

use clap::Parser;

#[derive(Parser)]
struct Cli {
    /// The CNF file with the DIMACS instance.
    file: PathBuf,
}

fn main() {
    env_logger::builder()
        .format(|buf, record| writeln!(buf, "c [{}] {}", record.level(), record.args()))
        .init();

    let cli = Cli::parse();

    match limiga::run_solver(cli.file) {
        Ok(Some(assignment)) => {
            println!("s SATISFIABLE");
            println!("v {}", assignment.value_line());
        }
        Ok(None) => {
            println!("s UNSATISFIABLE");
        }
        Err(e) => eprintln!("Error: {e}"),
    }
}
