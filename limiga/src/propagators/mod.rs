mod not_eq;

pub use not_eq::*;

use std::ops::{ControlFlow, FromResidual, Try};

/// After propagation, a domain may become empty. This enum is used to indicate whether a domain
/// has become empty.
pub enum PropagationResult {
    Consistent,
    Inconsistent,
}

/// A propagator removes values from domains which do not participate in any satisfying solution.
pub trait Propagator<VStore> {
    fn propagate(&mut self, store: &mut VStore) -> PropagationResult;
}

impl From<bool> for PropagationResult {
    fn from(value: bool) -> Self {
        if value {
            PropagationResult::Consistent
        } else {
            PropagationResult::Inconsistent
        }
    }
}

impl Try for PropagationResult {
    type Output = ();

    type Residual = PropagationResult;

    fn from_output(_: Self::Output) -> Self {
        PropagationResult::Consistent
    }

    fn branch(self) -> std::ops::ControlFlow<Self::Residual, Self::Output> {
        match self {
            PropagationResult::Consistent => ControlFlow::Continue(()),
            PropagationResult::Inconsistent => ControlFlow::Break(self),
        }
    }
}

impl FromResidual for PropagationResult {
    fn from_residual(_: <Self as Try>::Residual) -> Self {
        PropagationResult::Inconsistent
    }
}
