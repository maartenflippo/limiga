use crate::{
    domains::{DomainStore, Domains},
    propagators::Propagator,
    IntVar,
};

#[derive(Default)]
pub struct Solver {
    domains: Domains,
    propagators: Vec<Box<dyn Propagator<Domains>>>,
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
}
