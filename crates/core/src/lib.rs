pub mod assignment;
pub mod brancher;
pub mod clausal_propagator;
pub mod clause;
pub mod lit;
pub mod solver;
pub mod storage;

pub type SatSolver = solver::Solver<brancher::NaiveBrancher>;
