pub mod assignment;
pub mod brancher;
pub mod clause;
pub mod lit;
pub mod solver;
pub mod storage;
pub mod trail;

pub type SatSolver = solver::Solver<brancher::NaiveBrancher>;
