use std::ops::{Add, Sub};

use crate::{propagators::RegistrationContext, Variable};

/// A view that offsets the domain of the inner variable by a constant amount. It models the
/// constraint `x = y + c` where `x, y` are variables and `c` is a constant.
pub struct OffsetView<Var, Store, DomainRegistrar>
where
    Var: Variable<Store, DomainRegistrar>,
{
    inner: Var,
    offset: Var::Value,
}

impl<Var, Store, DomainRegistrar> OffsetView<Var, Store, DomainRegistrar>
where
    Var: Variable<Store, DomainRegistrar>,
{
    pub fn new(var: Var, offset: Var::Value) -> Self {
        OffsetView { inner: var, offset }
    }
}

impl<Var, Store, DomainRegistrar> Variable<Store, DomainRegistrar>
    for OffsetView<Var, Store, DomainRegistrar>
where
    Var: Variable<Store, DomainRegistrar>,
    for<'a> Var::Value: Add<&'a Var::Value, Output = Var::Value>,
    for<'a, 'b> &'a Var::Value: Sub<&'b Var::Value, Output = Var::Value>,
    DomainRegistrar: RegistrationContext<Var::Dom>,
{
    type Value = Var::Value;
    type Dom = Var::Dom;

    fn min(&self, store: &Store) -> Self::Value {
        self.inner.min(store) + &self.offset
    }

    fn max(&self, store: &Store) -> Self::Value {
        self.inner.max(store) + &self.offset
    }

    fn fixed_value(&self, store: &Store) -> Option<Self::Value> {
        self.inner
            .fixed_value(store)
            .map(|value| value + &self.offset)
    }

    fn remove(&self, store: &mut Store, value: &Self::Value) -> bool {
        let value = value.sub(&self.offset);
        self.inner.remove(store, &value)
    }

    fn set_min(&self, store: &mut Store, value: &Self::Value) -> bool {
        let value = value.sub(&self.offset);
        self.inner.set_min(store, &value)
    }

    fn set_max(&self, store: &mut Store, value: &Self::Value) -> bool {
        let value = value.sub(&self.offset);
        self.inner.set_max(store, &value)
    }

    fn register(&self, registrar: &mut DomainRegistrar) {
        self.inner.register(registrar);
    }
}

impl<Var, Store, DomainRegistrar> Clone for OffsetView<Var, Store, DomainRegistrar>
where
    Var: Variable<Store, DomainRegistrar> + Clone,
    Var::Value: Clone,
{
    fn clone(&self) -> Self {
        OffsetView::new(self.inner.clone(), self.offset.clone())
    }
}

impl<Var, Store, DomainRegistrar> Copy for OffsetView<Var, Store, DomainRegistrar>
where
    Var: Variable<Store, DomainRegistrar> + Clone + Copy,
    Var::Value: Clone + Copy,
{
}
