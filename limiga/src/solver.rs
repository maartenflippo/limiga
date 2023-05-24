use crate::{
    domains::{DomainId, DomainStore, Domains, GlobalDomainId},
    propagators::{PropagationResult, Propagator, PropagatorStore, RegistrationContext},
    IntVar, Variable,
};

#[derive(Default)]
pub struct Solver {
    domains: Domains,
    propagators: PropagatorStore<Domains, PropagatorRegistration>,
}

pub enum SolveOutcome<'solver> {
    Satisfiable(SolutionIterator<'solver>),
    Unsatisfiable,
}

pub struct SolutionIterator<'solver> {
    solver: &'solver Solver,
}

pub struct Solution<'solver> {
    solver: &'solver Solver,
}

pub struct PropagatorRegistration(Vec<GlobalDomainId>);

impl Solver {
    pub fn new_int_var<Dom>(&mut self, domain: Dom) -> IntVar<Dom, Domains, PropagatorRegistration>
    where
        Domains: DomainStore<Dom>,
    {
        self.domains.alloc(domain).into()
    }

    pub fn post(
        &mut self,
        propagator: Box<dyn Propagator<Domains, PropagatorRegistration>>,
    ) -> PropagationResult {
        let propagator_id = self.propagators.alloc(propagator);

        let mut ctx = PropagatorRegistration(vec![]);
        let propagator = self.propagators.get_mut(propagator_id);
        propagator.initialize(&mut ctx);

        propagator.propagate(&mut self.domains)
    }

    pub fn solve(&mut self) -> SolveOutcome<'_> {
        SolveOutcome::Unsatisfiable
    }
}

impl<'solver> SolutionIterator<'solver> {
    pub fn next<'a: 'solver>(&'a self) -> Option<Solution<'a>> {
        Some(Solution {
            solver: self.solver,
        })
    }
}

impl<'solver> Solution<'solver> {
    pub fn value<Var, DomainRegistrar>(&self, variable: Var) -> Var::Value
    where
        Var: Variable<Domains, DomainRegistrar>,
        Domains: DomainStore<Var::Dom>,
    {
        variable
            .fixed_value(&self.solver.domains)
            .expect("in a solution all variables are fixed")
    }
}

impl<Dom> RegistrationContext<Dom> for PropagatorRegistration {
    fn register(&mut self, domain: DomainId<Dom>) {
        self.0.push(domain.global_id());
    }
}
