use crate::{lit::Var, propagation::Reason, storage::KeyedVec};

#[derive(Default)]
pub struct ImplicationGraph<Domains> {
    reasons: KeyedVec<Var, Reason<Domains>>,
}

impl<Domains> ImplicationGraph<Domains> {
    pub fn grow_to(&mut self, var: Var) {
        self.reasons.grow_to(var);
    }

    /// Get the reason for the assignment of the given literal.
    pub fn reason(&self, var: Var) -> &Reason<Domains> {
        &self.reasons[var]
    }

    /// Add a reason to the implication graph for the given literal.
    pub fn add(&mut self, var: Var, reason: Reason<Domains>) {
        self.reasons[var] = reason;
    }
}
