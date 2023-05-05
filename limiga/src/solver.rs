use crate::{
    domains::{DomainStore, Domains},
    propagators::Propagator,
    IntVar, Variable,
};

#[derive(Default)]
pub struct Solver {
    domains: Domains,
    propagators: Vec<Box<dyn Propagator<Domains>>>,
}

pub enum SolveOutcome<'solver> {
    Satisfiable(Solution<'solver>),
    Unsatisfiable,
}

pub struct Solution<'solver> {
    solver: &'solver Solver,
}

impl Solver {
    pub fn new_int_var<Dom>(&mut self, domain: Dom) -> IntVar<Dom>
    where
        Domains: DomainStore<Dom>,
    {
        self.domains.alloc(domain).into()
    }

    pub fn post(&mut self, propagator: Box<dyn Propagator<Domains>>) {
        self.propagators.push(propagator);
    }

    pub fn solve(&mut self) -> SolveOutcome<'_> {
        SolveOutcome::Unsatisfiable
    }
}

impl<'solver> Solution<'solver> {
    pub fn value<Var>(&self, variable: Var) -> &Var::Value
    where
        Var: Variable<Domains>,
        Domains: DomainStore<Var::Dom>,
    {
        variable
            .fixed_value(&self.solver.domains)
            .expect("in a solution all variables are fixed")
    }
}
