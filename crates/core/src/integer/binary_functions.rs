#![allow(unused)]

use crate::{
    domains::{Conflict, Domain},
    propagation::{Context, Explanation},
    variable::Variable, atom::Atom,
};

use super::{BoundedIntVar, Int};

/// Models the relationship `z = min(x, y)` as a function on the domains of `x` and `y`.
#[derive(Clone)]
pub struct Min<VX, VY> {
    x: VX,
    y: VY,
}

impl<VX, VY> Min<VX, VY> {
    pub fn new(x: VX, y: VY) -> Self {
        Min { x, y }
    }
}

impl<VX, VY, Event> Variable for Min<VX, VY>
where
    VX: Variable,
    VY: Variable,
    VX::Dom: Domain<ProducedEvent = Event>,
    VY::Dom: Domain<ProducedEvent = Event>,
{
    type Dom = Min<VX, VY>;
}

impl<VX, VY, Event> Domain for Min<VX, VY>
where
    VX: Variable,
    VY: Variable,
    VX::Dom: Domain<ProducedEvent = Event>,
    VY::Dom: Domain<ProducedEvent = Event>,
{
    type ProducedEvent = Event;
}

impl<VX, VY, Domains, Event, VarEvent> BoundedIntVar<Domains, Event> for Min<VX, VY>
where
    VX: BoundedIntVar<Domains, Event>,
    VY: BoundedIntVar<Domains, Event>,
    VX::Dom: Domain<ProducedEvent = VarEvent>,
    VY::Dom: Domain<ProducedEvent = VarEvent>,
{
    fn max(&self, ctx: &mut Context<Domains, Event>) -> Int {
        Int::max(self.x.max(ctx), self.y.max(ctx))
    }

    fn min(&self, ctx: &mut Context<Domains, Event>) -> Int {
        Int::max(self.x.min(ctx), self.y.min(ctx))
    }

    fn upper_bound_atom(&self, bound: Int) -> Box<dyn Atom<Domains>> {
        todo!()
    }

    fn lower_bound_atom(&self, bound: Int) -> Box<dyn Atom<Domains>> {
        todo!()
    }

    fn set_min(
        &self,
        ctx: &mut Context<Domains, Event>,
        bound: Int,
        explanation: impl Into<Explanation<Domains>>,
    ) -> Result<(), Conflict<Domains>> {
        let explanation = explanation.into();

        self.x.set_min(ctx, bound, explanation.clone())?;
        self.y.set_min(ctx, bound, explanation)?;

        Ok(())
    }

    fn set_max(
        &self,
        ctx: &mut Context<Domains, Event>,
        bound: Int,
        explanation: impl Into<Explanation<Domains>>,
    ) -> Result<(), Conflict<Domains>> {
        todo!()
    }

}
