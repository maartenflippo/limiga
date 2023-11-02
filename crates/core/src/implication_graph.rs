use crate::{
    clause::ClauseRef,
    lit::{Lit, Var},
    storage::KeyedVec,
};

#[derive(Default)]
pub struct ImplicationGraph {
    reasons: KeyedVec<Lit, ClauseRef>,
}

impl ImplicationGraph {
    pub fn grow_to(&mut self, var: Var) {
        self.reasons.grow_to(Lit::positive(var));
    }

    /// Get the reason for the assignment of the given literal.
    pub fn reason(&self, lit: Lit) -> ClauseRef {
        self.reasons[lit]
    }

    /// Add a reason to the implication graph for the given literal.
    pub fn add(&mut self, lit: Lit, reason: ClauseRef) {
        self.reasons[lit] = reason;
    }
}
