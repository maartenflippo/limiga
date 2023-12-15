use std::{
    fmt::{Debug, Write},
    fs::File,
    num::NonZeroI32,
    path::Path,
    time::Duration,
};

use crate::{
    error::LimigaError,
    termination::{OrTerminator, SignalTerminator},
};
use limiga_core::{
    brancher::{Brancher, VsidsBrancher},
    lit::{Lit, Var},
    solver::{Solution, SolveResult, Solver},
    storage::{Indexer, StaticIndexer},
    termination::TimeBudget,
};
use limiga_dimacs::DimacsSink;

pub struct Assignment {
    values: Box<[bool]>,
}

pub enum Conclusion {
    Satisfiable(Assignment),
    Unsatisfiable,
    Unknown,
}

pub fn run_solver(
    path: impl AsRef<Path>,
    timeout: Option<Duration>,
) -> Result<Conclusion, LimigaError> {
    let file = File::open(path)?;
    let timer = timeout
        .map(TimeBudget::starting_now)
        .unwrap_or(TimeBudget::infinite());

    let signal_terminator = SignalTerminator::register();
    let terminator = OrTerminator::new(timer, signal_terminator);

    let mut solver: Solver<_, (), ()> = Solver::new(VsidsBrancher::new(0.95));
    let mut sink = limiga_dimacs::parse_cnf(file, |header| {
        let vars = solver
            .new_lits()
            .take(header.num_variables)
            .map(|lit| lit.var())
            .collect::<Vec<_>>()
            .into();

        SolverSink { solver, vars }
    })?;

    match sink.solver.solve(terminator) {
        SolveResult::Satisfiable(solution) => Ok(Conclusion::Satisfiable(solution.into())),
        SolveResult::Unsatisfiable => Ok(Conclusion::Unsatisfiable),
        SolveResult::Unknown => Ok(Conclusion::Unknown),
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

impl<'a> From<Solution<'a>> for Assignment {
    fn from(solution: Solution<'a>) -> Self {
        let values = solution.vars().map(|var| solution.value(var)).collect();

        Assignment { values }
    }
}

struct SolverSink<SearchProc, Domains, Event> {
    solver: Solver<SearchProc, Domains, Event>,
    vars: Box<[Var]>,
}

impl<SearchProc, Domains, Event> DimacsSink for SolverSink<SearchProc, Domains, Event>
where
    SearchProc: Brancher,
    Event: Copy + Debug + StaticIndexer,
{
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
