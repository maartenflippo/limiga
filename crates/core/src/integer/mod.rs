pub mod interval_domain;

use crate::{
    domains::{Domain, DomainId, DomainStore},
    propagation::{Context, Reason},
    variable::Variable,
    Conflict,
};

/// The type of integer variables we support.
pub type Int = i32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntEvent {
    LowerBound,
    UpperBound,
}

pub trait BoundedInt: Domain<ProducedEvent = IntEvent> {
    /// Get the upper bound of the domain.
    fn max(&self) -> Int;

    /// Tighten the lower bound of the domain to the new bound. If the given bound is smaller than
    /// the current lower bound, this is a no-op.
    fn set_min(&mut self, bound: Int) -> Result<(), Conflict>;
}

pub trait BoundedIntVar<Domains, Event>: Variable {
    /// Get the upper bound of the domain.
    fn max(&self, ctx: &mut Context<Domains, Event>) -> Int;

    /// Tighten the lower bound of the domain to the new bound. If the given bound is smaller than
    /// the current lower bound, this is a no-op.
    fn set_min(
        &self,
        ctx: &mut Context<Domains, Event>,
        bound: Int,
        reason: Reason,
    ) -> Result<(), Conflict>;
}

impl<Dom, Domains, Event> BoundedIntVar<Domains, Event> for DomainId<Dom>
where
    Dom: BoundedInt + 'static,
    Domains: DomainStore<Dom>,
{
    fn max(&self, ctx: &mut Context<Domains, Event>) -> Int {
        ctx.read(self.clone()).max()
    }

    fn set_min(
        &self,
        ctx: &mut Context<Domains, Event>,
        bound: Int,
        _reason: Reason,
    ) -> Result<(), Conflict> {
        ctx.read_mut(self.clone()).set_min(bound)
    }
}
