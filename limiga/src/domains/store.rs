use std::marker::PhantomData;

use super::BitSetDomain;

/// A handle to a domain.
pub struct DomainId<Dom> {
    index: usize,
    global_id: GlobalDomainId,
    dom: PhantomData<Dom>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlobalDomainId(usize);

#[derive(Default)]
pub struct Domains {
    bitsets: Vec<BitSetDomain>,
    next_global_id: usize,
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

        self.next_global_id += 1;

        DomainId {
            index: self.bitsets.len() - 1,
            global_id: GlobalDomainId(self.next_global_id),
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

impl<Dom> DomainId<Dom> {
    pub fn global_id(&self) -> GlobalDomainId {
        self.global_id
    }
}

impl<Dom> Clone for DomainId<Dom> {
    fn clone(&self) -> Self {
        DomainId {
            index: self.index,
            global_id: self.global_id,
            dom: PhantomData,
        }
    }
}

impl<Dom> Copy for DomainId<Dom> {}

#[cfg(test)]
mod tests {
    use crate::domains::Domain;

    use super::*;

    #[test]
    fn domain_id_returns_appropriate_domain_reference() {
        let mut store = Domains::default();

        let d1 = store.alloc(BitSetDomain::new(1, 10));
        let d2 = store.alloc(BitSetDomain::new(5, 50));
        assert_ne!(d1.global_id(), d2.global_id());

        let d1 = store.read(d1);
        let d2 = store.read(d2);

        assert_eq!(1, d1.min());
        assert_eq!(10, d1.max());

        assert_eq!(5, d2.min());
        assert_eq!(50, d2.max());
    }
}
