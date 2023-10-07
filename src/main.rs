use std::{fs::File, path::PathBuf};

use clap::Parser;
use limiga_core::{
    lit::{Lit, Var},
    solver::SolveResult,
    SatSolver,
};

use limiga_dimacs::DimacsSink;

#[derive(Parser)]
struct Cli {
    /// The CNF file with the DIMACS instance.
    file: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    let file = match File::open(&cli.file) {
        Ok(file) => file,
        Err(_) => {
            eprintln!("Failed to read file.");
            return;
        }
    };

    let mut solver = SatSolver::default();
    let sink = limiga_dimacs::parse_cnf(file, |header| {
        let vars = solver
            .new_lits()
            .take(header.num_variables)
            .map(|lit| lit.var())
            .collect::<Vec<_>>()
            .into();

        SolverSink { solver, vars }
    });

    let mut sink = match sink {
        Ok(sink) => sink,
        Err(e) => {
            eprintln!("Failed to parse DIMACS: {}", e);
            return;
        }
    };

    match sink.solver.solve() {
        SolveResult::Satisfiable(solution) => {
            println!("s SATISFIABLE");

            let solution_line = sink
                .vars
                .iter()
                .enumerate()
                .map(|(idx, var)| {
                    let dimacs_lit = idx + 1;

                    if solution.value(*var) {
                        format!("{dimacs_lit} ")
                    } else {
                        format!("-{dimacs_lit} ")
                    }
                })
                .collect::<String>();

            println!("v {solution_line}");
        }
        SolveResult::Unsatisfiable => {
            println!("s UNSATISFIABLE");
        }
    }
}

struct SolverSink {
    solver: SatSolver,
    vars: Box<[Var]>,
}

impl DimacsSink for SolverSink {
    fn add_clause(&mut self, lits: &[std::num::NonZeroI32]) {
        let lits = lits
            .iter()
            .map(|lit| {
                let idx = lit.get().unsigned_abs() as usize - 1;

                if lit.is_positive() {
                    Lit::positive(self.vars[idx])
                } else {
                    Lit::negative(self.vars[idx])
                }
            })
            .collect::<Vec<_>>();

        self.solver.add_clause(lits);
    }
}
