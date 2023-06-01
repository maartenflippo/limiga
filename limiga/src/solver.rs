use crate::{
    domains::{DomainId, DomainStore, Domains, GlobalDomainId},
    propagators::{PropagationResult, Propagator, PropagatorStore, RegistrationContext},
    search::Brancher,
    IntVar, Variable,
};

#[derive(Default)]
pub struct Solver {
    domains: Domains,
    propagators: PropagatorStore<Domains, PropagatorRegistration>,
}

pub enum SolveOutcome<'solver, Brancher> {
    Satisfiable(SolutionIterator<'solver, Brancher>),
    Unsatisfiable,
}

pub struct SolutionIterator<'solver, Brancher> {
    solver: &'solver mut Solver,
    brancher: Brancher,
    search_on_next: bool,
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

    pub fn solve<B: Brancher<Domains>>(&mut self, _brancher: B) -> SolveOutcome<'_, B> {
        SolveOutcome::Unsatisfiable
    }

    fn next_solution(&mut self, _brancher: &mut impl Brancher<Domains>) -> bool {
        false
    }
}

impl<'solver, B: Brancher<Domains>> SolutionIterator<'solver, B> {
    pub fn next<'a>(&'a mut self) -> Option<Solution<'a>> {
        if !self.search_on_next {
            self.search_on_next = true;

            return Some(Solution {
                solver: self.solver,
            });
        }

        if self.solver.next_solution(&mut self.brancher) {
            Some(Solution {
                solver: self.solver,
            })
        } else {
            None
        }
    }
}

impl<'solver> Solution<'solver> {
    pub fn value<Var>(&self, variable: Var) -> Var::Value
    where
        Var: Variable<Domains>,
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
