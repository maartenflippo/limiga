pub mod affine_view;
pub mod atoms;
mod binary_functions;
pub mod interval_domain;

use crate::{
    atom::Atom,
    domains::{Conflict, Domain, DomainId, DomainStore, EnqueueDomainLit},
    lit::Lit,
    propagation::{Context, Explanation},
    variable::Variable,
};

use self::atoms::{AtLeast, AtMost};

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

    /// Get the literal that asserts the given upper bound for this domain.
    fn upper_bound_lit(&self, bound: Int) -> Lit;

    /// Get the literal that asserts the given lower bound for this domain.
    fn lower_bound_lit(&self, bound: Int) -> Lit;

    /// Tighten the lower bound of the domain to the new bound. If the given bound is smaller than
    /// the current lower bound, this is a no-op.
    fn set_min<Domains>(
        &mut self,
        bound: Int,
        explanation: Explanation<Domains>,
        enqueue_lit: impl EnqueueDomainLit<Domains>,
    ) -> Result<(), Conflict<Domains>>;

    /// Tighten the upper bound of the domain to the new bound. If the given bound is larger than
    /// the current upper bound, this is a no-op.
    fn set_max<Domains>(
        &mut self,
        bound: Int,
        explanation: Explanation<Domains>,
        enqueue_lit: impl EnqueueDomainLit<Domains>,
    ) -> Result<(), Conflict<Domains>>;
}

pub trait BoundedIntVar<Domains, Event>: Variable {
    /// Get the upper bound of the domain.
    fn max(&self, ctx: &mut Context<Domains, Event>) -> Int;

    /// Get the lower bound of the domain.
    fn min(&self, ctx: &mut Context<Domains, Event>) -> Int;

    /// Get the atom asserting the given bound as the upper bound of this variable.
    fn upper_bound_atom(&self, bound: Int) -> Box<dyn Atom<Domains>>;

    /// Get the atom asserting the given bound as the lower bound of this variable.
    fn lower_bound_atom(&self, bound: Int) -> Box<dyn Atom<Domains>>;

    /// Tighten the lower bound of the domain to the new bound. If the given bound is smaller than
    /// the current lower bound, this is a no-op.
    fn set_min(
        &self,
        ctx: &mut Context<Domains, Event>,
        bound: Int,
        explanation: impl Into<Explanation<Domains>>,
    ) -> Result<(), Conflict<Domains>>;

    /// Tighten the upper bound of the domain to the new bound. If the given bound is larger than
    /// the current upper bound, this is a no-op.
    fn set_max(
        &self,
        ctx: &mut Context<Domains, Event>,
        bound: Int,
        explanation: impl Into<Explanation<Domains>>,
    ) -> Result<(), Conflict<Domains>>;
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

    fn upper_bound_atom(&self, bound: Int) -> Box<dyn Atom<Domains>> {
        Box::new(AtMost { domain: self.clone(), bound })
    }

    fn lower_bound_atom(&self, bound: Int) -> Box<dyn Atom<Domains>> {
        Box::new(AtLeast { domain: self.clone(), bound })
    }

    fn set_min(
        &self,
        ctx: &mut Context<Domains, Event>,
        bound: Int,
        explanation: impl Into<Explanation<Domains>>,
    ) -> Result<(), Conflict<Domains>> {
        let (dom, enqueue_lit) = ctx.read_mut(self.clone());
        dom.set_min(bound, explanation.into(), enqueue_lit)
    }

    fn set_max(
        &self,
        ctx: &mut Context<Domains, Event>,
        bound: Int,
        explanation: impl Into<Explanation<Domains>>,
    ) -> Result<(), Conflict<Domains>> {
        let (dom, enqueue_lit) = ctx.read_mut(self.clone());
        dom.set_max(bound, explanation.into(), enqueue_lit)
    }
}
