use std::marker::PhantomData;

use super::BitSetDomain;

/// A handle to a domain.
pub struct DomainId<Dom> {
    index: usize,
    dom: PhantomData<Dom>,
}

#[derive(Default)]
pub struct Domains {
    bitsets: Vec<BitSetDomain>,
}

/// A domain store is a container of domains. Once domains have been allocated to the store, they
/// can be retrieved with their ID.
pub trait DomainStore<Dom> {
    /// Allocate a new domain into the store.
    fn alloc(&mut self, domain: Dom) -> DomainId<Dom>;

    /// Get the domain from the store.
    fn read(&self, id: DomainId<Dom>) -> &Dom;

    /// Get the domain from the store.
    fn read_mut(&mut self, id: DomainId<Dom>) -> &mut Dom;
}

impl DomainStore<BitSetDomain> for Domains {
    fn alloc(&mut self, domain: BitSetDomain) -> DomainId<BitSetDomain> {
        self.bitsets.push(domain);

        DomainId {
            index: self.bitsets.len(),
            dom: PhantomData,
        }
    }

    fn read(&self, id: DomainId<BitSetDomain>) -> &BitSetDomain {
        &self.bitsets[id.index]
    }

    fn read_mut(&mut self, id: DomainId<BitSetDomain>) -> &mut BitSetDomain {
        &mut self.bitsets[id.index]
    }
}

impl<Dom> Clone for DomainId<Dom> {
    fn clone(&self) -> Self {
        DomainId {
            index: self.index,
            dom: PhantomData,
        }
    }
}

impl<Dom> Copy for DomainId<Dom> {}
