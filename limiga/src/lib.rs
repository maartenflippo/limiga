#![feature(try_trait_v2)]

pub mod domains;
pub mod propagators;
pub mod search;

mod propagator_store;
mod solver;
mod variables;
mod views;

pub use solver::*;
pub use variables::*;
pub use views::*;
