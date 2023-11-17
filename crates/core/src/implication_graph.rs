use crate::{lit::Var, propagation::Reason, storage::KeyedVec};

#[derive(Default)]
pub struct ImplicationGraph {
    reasons: KeyedVec<Var, Reason>,
}

impl ImplicationGraph {
    pub fn grow_to(&mut self, var: Var) {
        self.reasons.grow_to(var);
    }

    /// Get the reason for the assignment of the given literal.
    pub fn reason(&self, var: Var) -> &Reason {
        &self.reasons[var]
    }

    /// Add a reason to the implication graph for the given literal.
    pub fn add(&mut self, var: Var, reason: Reason) {
        self.reasons[var] = reason;
    }
}
