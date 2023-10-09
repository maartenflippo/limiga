use bitvec::vec::BitVec;
use log::trace;

use crate::lit::{Lit, Var};

#[derive(Default)]
pub struct Assignment {
    snapshot: BitVec,
}

impl Assignment {
    pub fn grow_to(&mut self, var: Var) {
        let new_len = var_to_idx(var) + 2;
        if self.snapshot.len() < new_len + 1 {
            self.snapshot.resize(new_len, false);
        }
    }

    pub fn value(&self, lit: Lit) -> Option<bool> {
        let idx = lit_to_idx(lit);

        if self.snapshot[idx] {
            Some(self.snapshot[idx + 1] == lit.is_positive())
        } else {
            None
        }
    }

    pub fn assign(&mut self, lit: Lit) {
        trace!("assigning {lit:?}");

        let idx = lit_to_idx(lit);

        self.snapshot.set(idx, true);
        self.snapshot.set(idx + 1, lit.is_positive());
    }

    pub fn unassign(&mut self, lit: Lit) {
        trace!("clearing {lit:?}");
        let idx = lit_to_idx(lit);
        self.snapshot.set(idx, false);
    }
}

#[inline]
fn lit_to_idx(lit: Lit) -> usize {
    var_to_idx(lit.var())
}

#[inline]
fn var_to_idx(var: Var) -> usize {
    var.code() as usize * 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initially_literals_are_unassigned() {
        let mut assignment = Assignment::default();
        assignment.grow_to(Var::try_from(3).unwrap());

        for code in 0..=3 {
            let lit = Lit::positive(code.try_into().unwrap());
            assert_eq!(None, assignment.value(lit));
            assert_eq!(None, assignment.value(!lit));
        }
    }

    #[test]
    fn an_assignment_can_be_queried() {
        let mut assignment = Assignment::default();
        assignment.grow_to(Var::try_from(3).unwrap());

        let lit = Lit::positive(Var::try_from(1).unwrap());
        assignment.assign(lit);

        assert_eq!(Some(true), assignment.value(lit));
        assert_eq!(Some(false), assignment.value(!lit));
    }
}

