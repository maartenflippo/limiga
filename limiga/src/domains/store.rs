use std::marker::PhantomData;

use crate::keyed_idx_vec::Key;

use super::{BitSetDomain, Domain, DomainMut};

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
    history: Vec<Vec<BitSetDomain>>,
    next_global_id: usize,
    updated_domains: Vec<GlobalDomainId>,
}

/// A domain store is a container of domains. Once domains have been allocated to the store, they
/// can be retrieved with their ID.
pub trait DomainStore<Dom> {
    /// Allocate a new domain into the store.
    fn alloc(&mut self, domain: Dom) -> DomainId<Dom>;

    /// Get the domain from the store.
    fn read<'dom>(&'dom self, id: DomainId<Dom>) -> DomainRef<'dom, Dom>;

    /// Get the domain from the store.
    fn read_mut<'dom>(&'dom mut self, id: DomainId<Dom>) -> DomainRefMut<'dom, Dom>;
}

pub struct DomainRef<'dom, Dom> {
    inner: &'dom Dom,
}

pub struct DomainRefMut<'dom, Dom> {
    inner: &'dom mut Dom,
    events: &'dom mut Vec<GlobalDomainId>,
    global_id: GlobalDomainId,
}

impl Domains {
    /// Save the current state to backtrack to later.
    pub fn push(&mut self) {
        self.history.push(self.bitsets.clone());
    }

    /// Return to the previously saved state.
    pub fn pop(&mut self) {
        if let Some(bitsets) = self.history.pop() {
            self.bitsets = bitsets;
        }
    }

    pub(crate) fn drain_updated_domains(&mut self) -> impl Iterator<Item = GlobalDomainId> + '_ {
        self.updated_domains.drain(..)
    }
}

impl<'dom, Dom: Domain> Domain for DomainRef<'dom, Dom> {
    type Value = Dom::Value;

    fn fixed_value(&self) -> Option<Self::Value> {
        self.inner.fixed_value()
    }

    fn min(&self) -> Self::Value {
        self.inner.min()
    }

    fn max(&self) -> Self::Value {
        self.inner.max()
    }

    fn size(&self) -> usize {
        self.inner.size()
    }
}

impl<'dom, Dom: Domain> Domain for DomainRefMut<'dom, Dom> {
    type Value = Dom::Value;

    fn fixed_value(&self) -> Option<Self::Value> {
        self.inner.fixed_value()
    }

    fn min(&self) -> Self::Value {
        self.inner.min()
    }

    fn max(&self) -> Self::Value {
        self.inner.max()
    }

    fn size(&self) -> usize {
        self.inner.size()
    }
}

impl<'dom, Dom: Domain> DomainRefMut<'dom, Dom> {
    fn wrap(&mut self, action: impl FnOnce(&mut Dom) -> bool) -> bool {
        let old_size = self.inner.size();
        let is_not_empty = action(self.inner);

        let new_size = self.inner.size();
        if is_not_empty {
            if old_size > new_size {
                self.events.push(self.global_id);
            }

            true
        } else {
            false
        }
    }
}

impl<'dom, Dom: DomainMut> DomainMut for DomainRefMut<'dom, Dom> {
    fn remove(&mut self, value: &Self::Value) -> bool {
        self.wrap(|dom| dom.remove(value))
    }

    fn set_max(&mut self, value: &Self::Value) -> bool {
        self.wrap(|dom| dom.set_max(value))
    }

    fn set_min(&mut self, value: &Self::Value) -> bool {
        self.wrap(|dom| dom.set_min(value))
    }

    fn fix(&mut self, value: &Self::Value) -> bool {
        self.wrap(|dom| dom.fix(value))
    }
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

    fn read<'dom>(&'dom self, id: DomainId<BitSetDomain>) -> DomainRef<'dom, BitSetDomain> {
        DomainRef {
            inner: &self.bitsets[id.index],
        }
    }

    fn read_mut<'dom>(
        &'dom mut self,
        id: DomainId<BitSetDomain>,
    ) -> DomainRefMut<'dom, BitSetDomain> {
        DomainRefMut {
            inner: &mut self.bitsets[id.index],
            events: &mut self.updated_domains,
            global_id: id.global_id,
        }
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

impl Key for GlobalDomainId {
    fn to_index(&self) -> usize {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::domains::Domain;

    use super::*;

    impl GlobalDomainId {
        pub fn from_index(idx: usize) -> GlobalDomainId {
            GlobalDomainId(idx)
        }
    }

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

    #[test]
    fn backtracking_restores_appropriate_state() {
        let mut store = Domains::default();

        let d1 = store.alloc(BitSetDomain::new(1, 10));

        store.push();
        {
            let mut dom = store.read_mut(d1);
            dom.set_min(&5);
            assert_eq!(5, dom.min());
        }

        store.pop();

        let dom = store.read(d1);
        assert_eq!(1, dom.min());
    }
}
