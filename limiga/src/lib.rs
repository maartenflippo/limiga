#![feature(try_trait_v2)]

pub mod domains;
pub mod propagators;

mod solver;
mod variables;

pub use solver::*;
pub use variables::*;
