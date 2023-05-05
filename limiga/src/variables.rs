use std::marker::PhantomData;

use crate::domains::{Domain, DomainId, DomainStore};

pub trait Variable<Store> {
    /// The type of the values for this variable.
    type Value: PartialOrd;

    /// The type of domain is attached to this variable.
    type Dom: Domain<Value = Self::Value>;

    /// Get the lower bound of the variable.
    fn min(&self, store: &Store) -> Self::Value;

    /// Get the upper bound of the variable.
    fn max(&self, store: &Store) -> Self::Value;

    /// Get the value if this variable has a singleton domain.
    fn fixed_value(&self, store: &Store) -> Option<Self::Value>;

    /// Remove the given value from this domain. If the domain becomes empty, this returns false.
    fn remove(&self, store: &mut Store, value: &Self::Value) -> bool;

    /// Set the lower bound for this variable.
    fn set_min(&self, store: &mut Store, value: &Self::Value) -> bool;

    /// Set the upper bound for this variable.
    fn set_max(&self, store: &mut Store, value: &Self::Value) -> bool;
}

pub struct IntVar<Dom, Store> {
    domain: DomainId<Dom>,
    store: PhantomData<Store>,
}

impl<Dom, Store> Variable<Store> for IntVar<Dom, Store>
where
    Dom: Domain<Value = i64> + 'static,
    Store: DomainStore<Dom>,
{
    type Value = i64;
    type Dom = Dom;

    fn min(&self, store: &Store) -> Self::Value {
        store.read(self.domain).min()
    }

    fn max(&self, store: &Store) -> Self::Value {
        store.read(self.domain).max()
    }

    fn fixed_value(&self, store: &Store) -> Option<Self::Value> {
        store.read(self.domain).fixed_value()
    }

    fn remove(&self, store: &mut Store, value: &Self::Value) -> bool {
        store.read_mut(self.domain).remove(value)
    }

    fn set_min(&self, store: &mut Store, value: &Self::Value) -> bool {
        store.read_mut(self.domain).set_min(value)
    }

    fn set_max(&self, store: &mut Store, value: &Self::Value) -> bool {
        store.read_mut(self.domain).set_max(value)
    }
}

impl<Dom, Store> From<DomainId<Dom>> for IntVar<Dom, Store> {
    fn from(value: DomainId<Dom>) -> Self {
        IntVar {
            domain: value,
            store: PhantomData,
        }
    }
}

impl<Dom, Store> Clone for IntVar<Dom, Store> {
    fn clone(&self) -> Self {
        IntVar {
            domain: self.domain,
            store: PhantomData,
        }
    }
}
impl<Dom, Store> Copy for IntVar<Dom, Store> {}
