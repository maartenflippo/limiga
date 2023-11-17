//! A solution to the `BIBD(v, b, r, k, l)` problem is a binary `v * b` matrix, whose rows sum to
//! `r`, whose columns sum to `k`. Any two distinct rows in the matrix can have a dot-product of
//! at-most `l`.

use limiga_constraints as constraints;
use limiga_core::{
    brancher::VsidsBrancher,
    domains::TypedDomainStore,
    integer::{interval_domain::IntInterval, Int, IntEvent},
    propagation::{LitEvent, SDomainEvent},
    solver::{SolveResult, Solver},
    storage::{Indexer, StaticIndexer},
    termination::Indefinite,
};

#[allow(clippy::upper_case_acronyms)]
struct BIBD {
    /// The number of rows in the matrix.
    rows: usize,
    /// The number of columns in the matrix.
    columns: usize,

    /// The value that every row in the matrix should sum to.
    row_sum: usize,
    /// The value that every column in the matrix should sum to.
    column_sum: usize,
    // /// The maximum overlap between any distinct pair of rows.
    // maximum_dot_product: u32,
}

impl BIBD {
    fn parse() -> Option<Self> {
        let [s1, s2, s3] = std::env::args()
            .skip(1)
            .take(3)
            .collect::<Vec<_>>()
            .try_into()
            .ok()?;

        let v = s1.parse::<usize>().ok()?;
        let k = s2.parse::<usize>().ok()?;
        let l = s3.parse::<usize>().ok()?;

        let r = l * (v - 1) / (k - 1);
        let b = v * r / k;

        Some(BIBD {
            rows: v,
            columns: b,
            row_sum: r,
            column_sum: k,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SolverEvent {
    LitEvent(LitEvent),
    IntEvent(IntEvent),
}

impl From<LitEvent> for SolverEvent {
    fn from(value: LitEvent) -> Self {
        SolverEvent::LitEvent(value)
    }
}

impl From<IntEvent> for SolverEvent {
    fn from(value: IntEvent) -> Self {
        SolverEvent::IntEvent(value)
    }
}

impl Indexer for SolverEvent {
    fn index(&self) -> usize {
        match *self {
            SolverEvent::LitEvent(LitEvent::FixedTrue) => 0,
            SolverEvent::LitEvent(LitEvent::FixedFalse) => 1,
            SolverEvent::IntEvent(IntEvent::LowerBound) => 2,
            SolverEvent::IntEvent(IntEvent::UpperBound) => 3,
        }
    }
}

impl SDomainEvent<LitEvent> for SolverEvent {
    fn is(self, event: LitEvent) -> bool {
        matches!(self, SolverEvent::LitEvent(e) if e == event)
    }
}

impl SDomainEvent<IntEvent> for SolverEvent {
    fn is(self, event: IntEvent) -> bool {
        matches!(self, SolverEvent::IntEvent(e) if e == event)
    }
}

impl StaticIndexer for SolverEvent {
    fn get_len() -> usize {
        4
    }
}

fn main() {
    env_logger::init();

    let Some(bibd) = BIBD::parse() else {
        eprintln!("Usage: {} <v> <k> <l>", std::env::args().next().unwrap());
        return;
    };

    let mut solver: Solver<_, TypedDomainStore<IntInterval>, SolverEvent> =
        Solver::new(VsidsBrancher::new(0.99));
    let matrix = (0..bibd.rows)
        .map(|_| solver.new_lits().take(bibd.columns).collect::<Box<[_]>>())
        .collect::<Box<[_]>>();

    // Constraint: Every row should sum to `bibd.row_sum`:
    let row_sum = solver.new_domain(IntInterval::factory(
        bibd.row_sum as Int,
        bibd.row_sum as Int,
    ));
    for row in matrix.iter() {
        constraints::bool_lin_eq(&mut solver, row.clone(), row_sum.clone());
    }

    // Constraint: Every column should sum to `bibd.column_sum`:
    let column_sum = solver.new_domain(IntInterval::factory(
        bibd.column_sum as Int,
        bibd.column_sum as Int,
    ));
    for row in transpose(&matrix).iter() {
        constraints::bool_lin_eq(&mut solver, row.clone(), column_sum.clone());
    }

    match solver.solve(Indefinite) {
        SolveResult::Satisfiable(solution) => {
            for row in matrix.iter() {
                row.iter()
                    .map(|&lit| solution.value(lit.var()))
                    .for_each(|value| if value { print!("*") } else { print!(".") });

                println!();
            }
        }

        SolveResult::Unsatisfiable => println!("Unsatisfiable"),
        SolveResult::Unknown => println!("Unknown"),
    }
}

fn transpose<T: Clone>(matrix: &[Box<[T]>]) -> Box<[Box<[T]>]> {
    let mut transposed = vec![vec![]; matrix[0].len()];

    for col in 0..matrix[0].len() {
        for row in matrix {
            transposed[col].push(row[col].clone());
        }
    }

    transposed
        .into_iter()
        .map(|vec| vec.into_boxed_slice())
        .collect()
}
