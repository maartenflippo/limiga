use std::{fmt::Write, fs::File, num::NonZeroI32, path::Path};

use limiga_core::{
    lit::{Lit, Var},
    solver::{Solution, SolveResult},
    SatSolver,
};
use limiga_dimacs::DimacsSink;
use thiserror::Error;

pub struct Assignment {
    values: Box<[bool]>,
}

#[derive(Debug, Error)]
pub enum LimigaError {
    #[error("error reading file")]
    Io(#[from] std::io::Error),

    #[error("failed to parse dimacs")]
    DimacsError(#[from] limiga_dimacs::DimacsParseError),
}

pub fn run_solver(path: impl AsRef<Path>) -> Result<Option<Assignment>, LimigaError> {
    let file = File::open(path)?;

    let mut solver = SatSolver::default();
    let mut sink = limiga_dimacs::parse_cnf(file, |header| {
        let vars = solver
            .new_lits()
            .take(header.num_variables)
            .map(|lit| lit.var())
            .collect::<Vec<_>>()
            .into();

        SolverSink { solver, vars }
    })?;

    match sink.solver.solve() {
        SolveResult::Satisfiable(solution) => Ok(Some(solution.into())),
        SolveResult::Unsatisfiable => Ok(None),
    }
}

impl Assignment {
    /// Get the value for the given dimacs literal. If the code is larger than the largest variable
    /// in the assignment, this will panic.
    pub fn value(&self, dimacs_lit: NonZeroI32) -> bool {
        let idx = dimacs_lit.unsigned_abs().get() as usize - 1;
        self.values[idx] == dimacs_lit.is_positive()
    }

    /// Get the string of dimacs literals, space separated and with the 0 sentinel value at the
    /// end.
    pub fn value_line(&self) -> String {
        let mut line =
            self.values
                .iter()
                .enumerate()
                .fold(String::new(), |mut buf, (idx, &value)| {
                    let var = idx as i32 + 1;
                    let value = if value {
                        var.to_string()
                    } else {
                        (-var).to_string()
                    };

                    write!(buf, "{value} ").unwrap();

                    buf
                });

        line.push('0');
        line
    }
}

impl<'a, B> From<Solution<'a, B>> for Assignment {
    fn from(solution: Solution<'a, B>) -> Self {
        let values = solution.vars().map(|var| solution.value(var)).collect();

        Assignment { values }
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
