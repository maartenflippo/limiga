mod not_eq;

pub use not_eq::*;

use std::ops::{ControlFlow, DerefMut, FromResidual, Try};

use crate::domains::DomainId;

/// After propagation, a domain may become empty. This enum is used to indicate whether a domain
/// has become empty.
pub enum PropagationResult {
    Consistent,
    Inconsistent,
}

/// A propagator removes values from domains which do not participate in any satisfying solution.
pub trait Propagator<VStore, DomainRegistrar> {
    /// Indicate for what domain changes this propagator needs to be enqueued.
    fn initialize(&mut self, ctx: &mut DomainRegistrar);

    /// Perform the reasoning to prune domains.
    fn propagate(&mut self, store: &mut VStore) -> PropagationResult;
}

pub trait RegistrationContext<Dom> {
    fn register(&mut self, domain: DomainId<Dom>);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PropagatorId(usize);

pub struct PropagatorStore<VStore, DomainRegistrar> {
    propagators: Vec<Box<dyn Propagator<VStore, DomainRegistrar>>>,
}

impl<VStore, DomainRegistrar> PropagatorStore<VStore, DomainRegistrar> {
    pub fn alloc(
        &mut self,
        propagator: Box<dyn Propagator<VStore, DomainRegistrar>>,
    ) -> PropagatorId {
        self.propagators.push(propagator);

        PropagatorId(self.propagators.len() - 1)
    }

    pub fn get_mut(&mut self, id: PropagatorId) -> &mut dyn Propagator<VStore, DomainRegistrar> {
        self.propagators[id.0].deref_mut()
    }
}

impl<VStore, DomainRegistrar> Default for PropagatorStore<VStore, DomainRegistrar> {
    fn default() -> Self {
        PropagatorStore {
            propagators: vec![],
        }
    }
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
