use std::{
    fmt::Debug,
    ops::{Index, IndexMut},
};

use crate::lit::Lit;

pub struct LongClause {
    head: [Lit; 2],
    lits: Box<[Lit]>,
}

impl LongClause {
    fn new(lits: impl AsRef<[Lit]>) -> LongClause {
        let lits = lits.as_ref();

        LongClause {
            head: [lits[0], lits[1]],
            lits: lits.into(),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Lit> {
        self.lits.iter()
    }

    pub fn lits(&self) -> &[Lit] {
        &self.lits
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.lits.len()
    }

    pub fn swap_head(&mut self) {
        self.head.swap(0, 1);
        self.lits.swap(0, 1);
    }

    pub fn swap(&mut self, idx1: usize, idx2: usize) {
        self.lits.swap(idx1, idx2);

        self.head[0] = self.lits[0];
        self.head[1] = self.lits[1];
    }
}

impl Index<usize> for LongClause {
    type Output = Lit;

    fn index(&self, index: usize) -> &Self::Output {
        if index < self.head.len() {
            &self.head[index]
        } else {
            &self.lits[index]
        }
    }
}

impl Debug for LongClause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:?}, {:?}", self.head[0], self.head[1])?;

        for lit in self.lits.iter() {
            write!(f, ", {lit:?}")?;
        }

        write!(f, "]")?;

        Ok(())
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

        ClauseRef(self.clauses.len() as u32 - 1)
    }

    pub fn add_explanation_clause(&mut self, lits: impl AsRef<[Lit]>) -> ClauseRef {
        let clause_ref = self.add_clause(lits);
        self.explanation_clauses.push(clause_ref);

        clause_ref
    }
}

impl Index<ClauseRef> for ClauseDb {
    type Output = LongClause;

    fn index(&self, index: ClauseRef) -> &Self::Output {
        &self.clauses[index.0 as usize]
    }
}

impl IndexMut<ClauseRef> for ClauseDb {
    fn index_mut(&mut self, index: ClauseRef) -> &mut Self::Output {
        &mut self.clauses[index.0 as usize]
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct ClauseRef(u32);

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
