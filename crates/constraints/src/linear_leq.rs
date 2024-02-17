use limiga_core::{
    domains::{Conflict, DomainStore},
    integer::{BoundedInt, BoundedIntVar, Int, IntEvent},
    propagation::{
        Context, DomainEvent, Propagator, PropagatorFactory, PropagatorVar, VariableRegistrar,
        Watchable,
    },
};

pub struct LinearLeqFactory<Var> {
    pub terms: Box<[Var]>,
    pub rhs: Int,
}

impl<Var, Domains, Event> PropagatorFactory<Domains, Event> for LinearLeqFactory<Var>
where
    Event: DomainEvent<IntEvent>,
    Var: BoundedIntVar<Domains, Event> + Watchable<TypedEvent = IntEvent>,
    Var::Dom: BoundedInt,
    Domains: DomainStore<Var::Dom>,
{
    fn create(
        self,
        registrar: &mut VariableRegistrar<'_, Event>,
    ) -> Box<dyn Propagator<Domains, Event>> {
        let terms: Box<[PropagatorVar<Var>]> = self
            .terms
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, x_i)| registrar.register(x_i, (i as u32).into(), IntEvent::LowerBound))
            .collect();

        Box::new(LinearLeq {
            terms,
            rhs: self.rhs,
        })
    }
}

/// A propagator for the constraint `\sum x_i <= rhs`, where `x_i` are integer literals and `rhs`
/// is a constant.
pub struct LinearLeq<Var> {
    terms: Box<[PropagatorVar<Var>]>,
    rhs: Int,
}

impl<Var, Domains, Event> Propagator<Domains, Event> for LinearLeq<Var>
where
    Event: DomainEvent<IntEvent>,
    Var: BoundedIntVar<Domains, Event>,
    Var::Dom: BoundedInt,
    Domains: DomainStore<Var::Dom>,
{
    fn propagate(&mut self, ctx: &mut Context<Domains, Event>) -> Result<(), Conflict> {
        let optimistic_lhs = self.terms.iter().map(|term| term.min(ctx)).sum::<Int>();
        let mut explanation_base = self
            .terms
            .iter()
            .map(|term| term.min_lit(ctx))
            .collect::<Vec<_>>();

        for (idx, term) in self.terms.iter().enumerate() {
            let term_lb = term.min(ctx);
            let new_max = self.rhs - optimistic_lhs - term_lb;

            let min_lit = explanation_base.swap_remove(idx);

            term.set_max(ctx, new_max, explanation_base.clone())?;

            explanation_base.push(min_lit);
            let len = explanation_base.len() - 1;
            explanation_base.swap(idx, len);
        }

        todo!()
    }
}
