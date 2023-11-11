use crate::domains::{Domain, DomainId};

pub trait Variable: Clone + 'static {
    /// The type of the underlying domain of the variable.
    type Dom: Domain;
}

impl<Dom: Domain + 'static> Variable for DomainId<Dom> {
    type Dom = Dom;
}
