use crate::domains::{Domain, DomainId, DomainStore};

pub trait Variable<Store> {
    /// The type of the values for this variable.
    type Value: PartialOrd;

    /// The type of domain is attached to this variable.
    type Dom: Domain<Value = Self::Value>;

    /// Get the lower bound of the variable.
    fn min<'store>(&self, store: &'store Store) -> &'store Self::Value;

    /// Get the upper bound of the variable.
    fn max<'store>(&self, store: &'store Store) -> &'store Self::Value;

    /// Get the value if this variable has a singleton domain.
    fn fixed_value<'store>(&self, store: &'store Store) -> Option<&'store Self::Value>;

    /// Remove the given value from this domain. If the domain becomes empty, this returns false.
    fn remove(&self, store: &mut Store, value: &Self::Value) -> bool;

    /// Set the lower bound for this variable.
    fn set_min(&self, store: &mut Store, value: &Self::Value) -> bool;

    /// Set the upper bound for this variable.
    fn set_max(&self, store: &mut Store, value: &Self::Value) -> bool;
}

pub struct IntVar<Dom>(DomainId<Dom>);

impl<Dom, Store> Variable<Store> for IntVar<Dom>
where
    Dom: Domain<Value = i64> + 'static,
    Store: DomainStore<Dom>,
{
    type Value = i64;
    type Dom = Dom;

    fn min<'store>(&self, store: &'store Store) -> &'store Self::Value {
        store.read(self.0).min()
    }

    fn max<'store>(&self, store: &'store Store) -> &'store Self::Value {
        store.read(self.0).max()
    }

    fn fixed_value<'store>(&self, store: &'store Store) -> Option<&'store Self::Value> {
        store.read(self.0).fixed_value()
    }

    fn remove(&self, store: &mut Store, value: &Self::Value) -> bool {
        store.read_mut(self.0).remove(value)
    }

    fn set_min(&self, store: &mut Store, value: &Self::Value) -> bool {
        store.read_mut(self.0).set_min(value)
    }

    fn set_max(&self, store: &mut Store, value: &Self::Value) -> bool {
        store.read_mut(self.0).set_max(value)
    }
}

impl<Dom> From<DomainId<Dom>> for IntVar<Dom> {
    fn from(value: DomainId<Dom>) -> Self {
        IntVar(value)
    }
}

impl<Dom> Clone for IntVar<Dom> {
    fn clone(&self) -> Self {
        IntVar(self.0)
    }
}
impl<Dom> Copy for IntVar<Dom> {}
