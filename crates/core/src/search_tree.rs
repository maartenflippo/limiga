use crate::{
    lit::{Lit, Var},
    storage::KeyedVec,
};

#[derive(Default)]
pub struct SearchTree {
    current_depth: usize,
    decided_at: KeyedVec<Var, usize>,
}

impl SearchTree {
    /// Get the depth of the search tree.
    pub fn depth(&self) -> usize {
        self.current_depth
    }

    /// Get the decision level at which the given variable was assigned. In case the variable is
    /// not yet assigned, this will return stale data.
    pub fn decision_level(&self, var: Var) -> usize {
        self.decided_at[var]
    }

    /// Indicates whether the search is at the root of the search tree.
    pub fn is_at_root(&self) -> bool {
        self.depth() == 0
    }

    /// Register that a literal has been assigned at the current depth.
    pub fn register_assignment(&mut self, lit: Lit) {
        self.decided_at[lit.var()] = self.current_depth;
    }

    /// Cut the search tree to the new depth.
    pub fn cut(&mut self, depth: usize) {
        self.current_depth = depth;
    }

    /// Adds a new branch to the tree, increasing the depth by one.
    pub fn branch(&mut self) {
        self.current_depth += 1;
    }

    pub fn grow_to(&mut self, var: Var) {
        self.decided_at.grow_to(var);
    }
}
