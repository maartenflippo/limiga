use crate::domains::Domain;

pub trait Variable<Store> {
    /// The type of the values for this variable.
    type Value;

    type Dom: Domain<Value = Self::Value>;

    /// Read the domain from the store that belongs to this variable.
    fn domain<'store>(&self, store: &'store Store) -> &'store Self::Dom;

    /// Read the domain from the store that belongs to this variable, with the ability to mutate it.
    fn domain_mut<'store>(&self, store: &'store mut Store) -> &'store mut Self::Dom;
}
