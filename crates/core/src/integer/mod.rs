pub mod interval_domain;
pub mod affine_view;

use crate::{
    domains::{Conflict, Domain, DomainId, DomainStore, EnqueueDomainLit},
    lit::Lit,
    propagation::{Context, Explanation},
    variable::Variable,
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

    /// Get the lower bound of the domain.
    fn min(&self) -> Int;

    /// Get the literal that asserts the current upper bound for this domain.
    fn max_lit(&self) -> Lit;

    /// Get the literal that asserts the current lower bound for this domain.
    fn min_lit(&self) -> Lit;

    /// Tighten the lower bound of the domain to the new bound. If the given bound is smaller than
    /// the current lower bound, this is a no-op.
    fn set_min(
        &mut self,
        bound: Int,
        explanation: Explanation,
        enqueue_lit: impl EnqueueDomainLit,
    ) -> Result<(), Conflict>;

    /// Tighten the upper bound of the domain to the new bound. If the given bound is larger than
    /// the current upper bound, this is a no-op.
    fn set_max(
        &mut self,
        bound: Int,
        explanation: Explanation,
        enqueue_lit: impl EnqueueDomainLit,
    ) -> Result<(), Conflict>;
}

pub trait BoundedIntVar<Domains, Event>: Variable {
    /// Get the upper bound of the domain.
    fn max(&self, ctx: &mut Context<Domains, Event>) -> Int;

    /// Get the lower bound of the domain.
    fn min(&self, ctx: &mut Context<Domains, Event>) -> Int;

    /// Get the literal that asserts the current upper bound for this variable's domain.
    fn max_lit(&self, ctx: &mut Context<Domains, Event>) -> Lit;

    /// Get the literal that asserts the current lower bound for this variable's domain.
    fn min_lit(&self, ctx: &mut Context<Domains, Event>) -> Lit;

    /// Tighten the lower bound of the domain to the new bound. If the given bound is smaller than
    /// the current lower bound, this is a no-op.
    fn set_min(
        &self,
        ctx: &mut Context<Domains, Event>,
        bound: Int,
        explanation: impl Into<Explanation>,
    ) -> Result<(), Conflict>;

    /// Tighten the upper bound of the domain to the new bound. If the given bound is larger than
    /// the current upper bound, this is a no-op.
    fn set_max(
        &self,
        ctx: &mut Context<Domains, Event>,
        bound: Int,
        explanation: impl Into<Explanation>,
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

    fn min(&self, ctx: &mut Context<Domains, Event>) -> Int {
        ctx.read(self.clone()).min()
    }

    fn max_lit(&self, ctx: &mut Context<Domains, Event>) -> Lit {
        ctx.read(self.clone()).max_lit()
    }

    fn min_lit(&self, ctx: &mut Context<Domains, Event>) -> Lit {
        ctx.read(self.clone()).min_lit()
    }

    fn set_min(
        &self,
        ctx: &mut Context<Domains, Event>,
        bound: Int,
        explanation: impl Into<Explanation>,
    ) -> Result<(), Conflict> {
        let (dom, enqueue_lit) = ctx.read_mut(self.clone());
        dom.set_min(bound, explanation.into(), enqueue_lit)
    }

    fn set_max(
        &self,
        ctx: &mut Context<Domains, Event>,
        bound: Int,
        explanation: impl Into<Explanation>,
    ) -> Result<(), Conflict> {
        let (dom, enqueue_lit) = ctx.read_mut(self.clone());
        dom.set_max(bound, explanation.into(), enqueue_lit)
    }
}
