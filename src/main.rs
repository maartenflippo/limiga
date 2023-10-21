use std::{io::Write, path::PathBuf, time::Duration};

use clap::Parser;
use limiga::Conclusion;

#[derive(Parser)]
struct Cli {
    /// The CNF file with the DIMACS instance.
    file: PathBuf,

    /// The timeout of the solver in seconds.
    #[arg(short, long)]
    timeout: Option<u64>,
}

fn main() {
    env_logger::builder()
        .format(|buf, record| writeln!(buf, "c [{}] {}", record.level(), record.args()))
        .init();

    let cli = Cli::parse();
    let timeout = cli.timeout.map(Duration::from_secs);

    match limiga::run_solver(cli.file, timeout) {
        Ok(Conclusion::Satisfiable(assignment)) => {
            println!("s SATISFIABLE");
            println!("v {}", assignment.value_line());
        }
        Ok(Conclusion::Unsatisfiable) => {
            println!("s UNSATISFIABLE");
        }
        Ok(Conclusion::Unknown) => {
            println!("s UNKNOWN");
        }
        Err(e) => eprintln!("Error: {e}"),
    }
}
