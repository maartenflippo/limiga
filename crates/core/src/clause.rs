use std::{
    fmt::Debug,
    ops::{Index, IndexMut},
};

use crate::lit::Lit;

pub struct LongClause(Box<[Lit]>);

impl LongClause {
    fn new(lits: impl AsRef<[Lit]>) -> LongClause {
        let lits = lits.as_ref();

        LongClause(lits.into())
    }

    pub fn iter(&self) -> impl Iterator<Item = &Lit> {
        self.0.iter()
    }

    pub fn lits(&self) -> &[Lit] {
        &self.0
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn swap(&mut self, idx1: usize, idx2: usize) {
        self.0.swap(idx1, idx2);
    }
}

impl Index<usize> for LongClause {
    type Output = Lit;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl Debug for LongClause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

#[derive(Default)]
pub struct ClauseDb {
    clauses: Vec<LongClause>,
    explanation_clauses: Vec<ClauseRef>,
}

impl ClauseDb {
    pub fn add_clause(&mut self, lits: impl AsRef<[Lit]>) -> ClauseRef {
        assert!(
            lits.as_ref().len() > 1,
            "The clause db cannot add the empty clause or a unit clause."
        );

        let clause = LongClause::new(lits);
        self.clauses.push(clause);

        ClauseRef {
            index: self.clauses.len() as u32 - 1,
            is_learned: false,
        }
    }

    pub fn add_learned_clause(&mut self, lits: impl AsRef<[Lit]>) -> ClauseRef {
        let mut clause_ref = self.add_clause(lits);
        clause_ref.is_learned = true;

        clause_ref
    }

    pub fn add_explanation_clause(&mut self, lits: impl AsRef<[Lit]>) -> ClauseRef {
        let clause_ref = self.add_clause(lits);
        self.explanation_clauses.push(clause_ref);

        clause_ref
    }

    pub fn is_learned(&self, clause_ref: ClauseRef) -> bool {
        clause_ref.is_learned
    }
}

impl Index<ClauseRef> for ClauseDb {
    type Output = LongClause;

    fn index(&self, clause_ref: ClauseRef) -> &Self::Output {
        &self.clauses[clause_ref.index as usize]
    }
}

impl IndexMut<ClauseRef> for ClauseDb {
    fn index_mut(&mut self, clause_ref: ClauseRef) -> &mut Self::Output {
        &mut self.clauses[clause_ref.index as usize]
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ClauseRef {
    index: u32,
    is_learned: bool,
}

#[cfg(test)]
mod tests {
    use crate::lit;

    use super::*;

    #[test]
    fn a_long_clause_is_correctly_iterated() {
        let lits = unsafe { [lit!(1), lit!(2), lit!(-3)] };
        let clause = LongClause::new(lits);

        assert_eq!(lits.to_vec(), clause.iter().copied().collect::<Vec<_>>());
    }
}
