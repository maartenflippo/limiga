mod error;

use std::{io::Write, path::PathBuf, process::ExitCode, time::Duration};

use clap::Parser;

#[derive(Parser)]
struct Cli {
    /// The CNF file with the DIMACS instance.
    file: PathBuf,

    /// The timeout of the solver in seconds.
    #[arg(short, long)]
    timeout: Option<u64>,
}

fn main() -> ExitCode {
    env_logger::builder()
        .format(|buf, record| writeln!(buf, "c [{}] {}", record.level(), record.args()))
        .init();

    let cli = Cli::parse();
    let timeout = cli.timeout.map(Duration::from_secs);

    match cli.file.extension() {
        Some(ext) if ext == "cnf" => limiga::solve_cnf(cli.file, timeout),
        Some(ext) if ext == "fzn" => limiga::flatzinc::solve(cli.file, timeout),

        Some(_) | None => {
            eprintln!(
                "The file type of '{}' is not supported.",
                cli.file.display()
            );
            ExitCode::FAILURE
        }
    }
}
