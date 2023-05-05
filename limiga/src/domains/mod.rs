mod bitset;
mod store;

pub use bitset::*;
pub use store::*;

/// A domain describes the possible set of values for a variable. Domains must be finite, and the
/// elements must form a partial order. As a consequence, each domain must have an upper and lower
/// bound.
pub trait Domain {
    /// The type of value this domain contains.
    type Value: PartialOrd;

    /// If the domain is singleton, return the value. Otherwise, return [`None`].
    fn fixed_value(&self) -> Option<&Self::Value>;

    /// The lower bound of the domain.
    fn min(&self) -> &Self::Value;

    /// The upper bound of the domain.
    fn max(&self) -> &Self::Value;

    /// The number of elements in the domain. This is at most the difference between the upper and
    /// lower bound, but elements in between might be missing.
    fn size(&self) -> usize;

    /// Remove a value from this domain. If the domain becomes empty, false is returned. Otherwise,
    /// true is returned.
    fn remove(&mut self, value: &Self::Value) -> bool;

    /// Remove all values above the provided value, such that it is the upper bound of the domain.
    fn set_max(&mut self, value: &Self::Value);

    /// Remove all values below the provided value, such that it is the lower bound of the domain.
    fn set_min(&mut self, value: &Self::Value);
}
