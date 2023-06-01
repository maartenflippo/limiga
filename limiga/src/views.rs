use std::{
    marker::PhantomData,
    ops::{Add, Sub},
};

use crate::{Register, Variable};

/// A view that offsets the domain of the inner variable by a constant amount. It models the
/// constraint `x = y + c` where `x, y` are variables and `c` is a constant.
pub struct OffsetView<Var, Value, Store, Registrar> {
    inner: Var,
    offset: Value,
    registrar: PhantomData<Registrar>,
    store: PhantomData<Store>,
}

impl<Var, Value, Store, Registrar> OffsetView<Var, Value, Store, Registrar> {
    pub fn new(var: Var, offset: Value) -> Self {
        OffsetView {
            inner: var,
            offset,
            registrar: PhantomData,
            store: PhantomData,
        }
    }
}

impl<Var, Value, Store, Registrar> Variable<Store> for OffsetView<Var, Value, Store, Registrar>
where
    Var: Variable<Store>,
    for<'a> Var::Value: Add<&'a Value, Output = Var::Value>,
    for<'a, 'b> &'a Var::Value: Sub<&'b Value, Output = Var::Value>,
{
    type Value = Var::Value;
    type Dom = Var::Dom;

    fn min(&self, store: &Store) -> Self::Value {
        self.inner.min(store) + &self.offset
    }

    fn max(&self, store: &Store) -> Self::Value {
        self.inner.max(store) + &self.offset
    }

    fn size(&self, store: &Store) -> usize {
        self.inner.size(store)
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

    fn fix(&self, store: &mut Store, value: &Self::Value) -> bool {
        let value = value.sub(&self.offset);
        self.inner.fix(store, &value)
    }
}

impl<Var, Value, Store, Registrar> Register<Registrar> for OffsetView<Var, Value, Store, Registrar>
where
    Var: Register<Registrar>,
{
    fn register(&self, registrar: &mut Registrar) {
        self.inner.register(registrar);
    }
}

impl<Var, Value, Store, Registrar> Clone for OffsetView<Var, Value, Store, Registrar>
where
    Var: Clone,
    Value: Clone,
{
    fn clone(&self) -> Self {
        OffsetView::new(self.inner.clone(), self.offset.clone())
    }
}
