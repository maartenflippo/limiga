use std::{
    marker::PhantomData,
    ops::{Index, IndexMut},
};

pub trait Domain {
    /// The event type this domain produces when it is mutated.
    type ProducedEvent;
}

pub trait DomainStore<Domain>:
    Index<DomainId<Domain>, Output = Domain> + IndexMut<DomainId<Domain>>
{
    fn alloc(&mut self, domain: Domain) -> DomainId<Domain>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UntypedDomainId(usize);

pub struct DomainId<Domain> {
    domain: PhantomData<Domain>,
    index: usize,
}

impl<Domain> Clone for DomainId<Domain> {
    fn clone(&self) -> Self {
        DomainId {
            domain: PhantomData,
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
    fn alloc(&mut self, domain: Domain) -> DomainId<Domain> {
        self.domains.push(domain);

        DomainId {
            domain: PhantomData,
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
