use std::ops::Index;

use bitvec::vec::BitVec;

use crate::{
    clause::ClauseRef,
    lit::{Lit, Var},
    storage::KeyedVec,
};

#[derive(Default)]
pub struct Trail {
    trail: Vec<Lit>,
    decision_levels: Vec<usize>,

    snapshot: BitVec,
    reasons: KeyedVec<Lit, ClauseRef>,
    decided_at_level: KeyedVec<Var, usize>,
}

impl Trail {
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.trail.len()
    }

    pub fn grow_to(&mut self, var: Var) {
        let new_len = (var.code() as usize + 1) * 2;
        if new_len >= self.snapshot.len() {
            self.snapshot.resize(new_len, false);
            self.reasons.grow_to(Lit::positive(var));
            self.decided_at_level.grow_to(var);
        }
    }

    pub fn enqueue(&mut self, lit: Lit, reason: ClauseRef) -> bool {
        let snapshot_idx = lit_to_idx(lit);
        if self.snapshot[snapshot_idx] && self.snapshot[snapshot_idx + 1] != lit.is_positive() {
            // Conflicting assignment.
            return false;
        }

        self.trail.push(lit);
        self.snapshot.set(snapshot_idx, true);
        self.snapshot.set(snapshot_idx + 1, lit.is_positive());

        self.decided_at_level[lit.var()] = self.decision_levels.len();
        self.reasons[lit] = reason;

        true
    }

    pub fn value(&self, lit: Lit) -> Option<bool> {
        let snapshot_idx = lit_to_idx(lit);

        if self.snapshot[snapshot_idx] {
            Some(self.snapshot[snapshot_idx + 1] == lit.is_positive())
        } else {
            None
        }
    }

    pub fn new_decision_level(&mut self) {
        self.decision_levels.push(self.trail.len());
    }

    pub fn backtrack_to(&mut self, decision_level: usize) {
        println!("c backtracking to {decision_level}");
        println!("c   current trail = {:?}", self.trail);

        let trail_len = self.decision_levels[decision_level];
        println!("c   trail len at new decision level = {trail_len}");

        for idx in trail_len..self.trail.len() {
            let lit = self.trail[idx];
            let snapshot_idx = lit.var().code() as usize * 2;
            self.snapshot.set(snapshot_idx, false);
        }
    }

    /// Get the clause that caused the given lit to propagate. If the given lit was not propagated,
    /// calling this will return stale data.
    pub fn get_reason(&self, lit: Lit) -> ClauseRef {
        self.reasons[lit]
    }

    pub fn get_decision_level(&self, lit: Lit) -> usize {
        self.decided_at_level[lit.var()]
    }

    pub fn pop(&mut self) {
        let lit = self.trail.pop().unwrap();
        self.snapshot.set(lit_to_idx(lit), false);
    }

    pub fn last(&self) -> Option<Lit> {
        self.trail.last().copied()
    }
}

impl Index<usize> for Trail {
    type Output = Lit;

    fn index(&self, index: usize) -> &Self::Output {
        &self.trail[index]
    }
}

#[inline]
fn lit_to_idx(lit: Lit) -> usize {
    lit.var().code() as usize * 2
}

impl AsRef<[Lit]> for Trail {
    fn as_ref(&self) -> &[Lit] {
        &self.trail
    }
}
