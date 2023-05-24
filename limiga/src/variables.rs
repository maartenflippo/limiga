use std::marker::PhantomData;

use crate::{
    domains::{Domain, DomainId, DomainStore},
    propagators::RegistrationContext,
};

pub trait Variable<Store, DomainRegistrar> {
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

    /// Register the domain IDs this variable depends on with the registrar.
    fn register(&self, registrar: &mut DomainRegistrar);
}

pub struct IntVar<Dom, Store, DomainRegistrar> {
    domain: DomainId<Dom>,
    store: PhantomData<Store>,
    registrar: PhantomData<DomainRegistrar>,
}

impl<Dom, Store, DomainRegistrar> Variable<Store, DomainRegistrar>
    for IntVar<Dom, Store, DomainRegistrar>
where
    Dom: Domain<Value = i64> + 'static,
    Store: DomainStore<Dom>,
    DomainRegistrar: RegistrationContext<Dom>,
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

    fn register(&self, registrar: &mut DomainRegistrar) {
        registrar.register(self.domain);
    }
}

impl<Dom, Store, DomainRegistrar> From<DomainId<Dom>> for IntVar<Dom, Store, DomainRegistrar> {
    fn from(value: DomainId<Dom>) -> Self {
        IntVar {
            domain: value,
            store: PhantomData,
            registrar: PhantomData,
        }
    }
}

impl<Dom, Store, DomainRegistrar> Clone for IntVar<Dom, Store, DomainRegistrar> {
    fn clone(&self) -> Self {
        IntVar::from(self.domain)
    }
}
impl<Dom, Store, DomainRegistrar> Copy for IntVar<Dom, Store, DomainRegistrar> {}
