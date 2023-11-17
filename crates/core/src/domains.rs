use std::{
    marker::PhantomData,
    ops::{Index, IndexMut},
};

use crate::{lit::Lit, propagation::Explanation, solver::ExtendClausalSolver, storage::Indexer};

/// A domain factory creates a variable domain, and links it to the appropriate literals in the
/// solver.
pub trait DomainFactory<Event> {
    type Domain;
    fn create(self, clausal_solver: &mut impl ExtendClausalSolver<Event>) -> Self::Domain;
}

pub trait Domain {
    /// The event type this domain produces when it is mutated.
    type ProducedEvent;
}

pub trait EnqueueDomainLit {
    fn enqueue(&mut self, lit: Lit, explanation: Explanation) -> Result<(), Conflict>;
}

impl<F> EnqueueDomainLit for F
where
    F: Fn(Lit, Explanation) -> Result<(), Conflict>,
{
    fn enqueue(&mut self, lit: Lit, explanation: Explanation) -> Result<(), Conflict> {
        self(lit, explanation)
    }
}

pub trait DomainStore<Domain>:
    Index<DomainId<Domain>, Output = Domain> + IndexMut<DomainId<Domain>>
{
    fn alloc(&mut self, untyped_id: UntypedDomainId, domain: Domain) -> DomainId<Domain>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UntypedDomainId(usize);

impl Indexer for UntypedDomainId {
    fn index(&self) -> usize {
        self.0
    }
}

#[derive(Default)]
pub struct GlobalDomainIdPool(usize);

impl GlobalDomainIdPool {
    pub fn next_id(&mut self) -> UntypedDomainId {
        let id = UntypedDomainId(self.0);
        self.0 += 1;
        id
    }
}

pub struct DomainId<Domain> {
    domain: PhantomData<Domain>,
    pub untyped_id: UntypedDomainId,
    index: usize,
}

impl<Domain> Clone for DomainId<Domain> {
    fn clone(&self) -> Self {
        DomainId {
            domain: PhantomData,
            untyped_id: self.untyped_id,
            index: self.index,
        }
    }
}

pub struct TypedDomainStore<Domain> {
    domains: Vec<Domain>,
}

impl<Domain> Default for TypedDomainStore<Domain> {
    fn default() -> Self {
        TypedDomainStore { domains: vec![] }
    }
}

impl<Domain> DomainStore<Domain> for TypedDomainStore<Domain> {
    fn alloc(&mut self, untyped_id: UntypedDomainId, domain: Domain) -> DomainId<Domain> {
        self.domains.push(domain);

        DomainId {
            domain: PhantomData,
            untyped_id,
            index: self.domains.len() - 1,
        }
    }
}

impl<Domain> Index<DomainId<Domain>> for TypedDomainStore<Domain> {
    type Output = Domain;

    fn index(&self, index: DomainId<Domain>) -> &Self::Output {
        &self.domains[index.index]
    }
}

impl<Domain> IndexMut<DomainId<Domain>> for TypedDomainStore<Domain> {
    fn index_mut(&mut self, index: DomainId<Domain>) -> &mut Self::Output {
        &mut self.domains[index.index]
    }
}

#[derive(Debug)]
pub struct Conflict {
    pub lit: Lit,
    pub explanation: Explanation,
}
