use std::ops::{Index, IndexMut};

use crate::lit::Lit;

pub struct LongClause {
    pub head: [Lit; 2],
    pub tail: Box<[Lit]>,
}

impl LongClause {
    fn new(lits: impl AsRef<[Lit]>) -> LongClause {
        let lits = lits.as_ref();
        let tail: Box<[Lit]> = lits[2..].to_vec().into();

        LongClause {
            head: [lits[0], lits[1]],
            tail,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Lit> {
        self.head.iter().chain(self.tail.iter())
    }

    pub fn len(&self) -> usize {
        self.head.len() + self.tail.len()
    }
}

impl Index<usize> for LongClause {
    type Output = Lit;

    fn index(&self, index: usize) -> &Self::Output {
        if index < self.head.len() {
            &self.head[index]
        } else {
            &self.tail[index - self.head.len()]
        }
    }
}

#[derive(Default)]
pub struct ClauseDb {
    clauses: Vec<LongClause>,
}

impl ClauseDb {
    pub fn add_clause(&mut self, lits: &[Lit]) -> ClauseRef {
        assert!(
            lits.len() > 1,
            "The clause db cannot add the empty clause or a unit clause."
        );

        let clause = LongClause::new(lits);
        self.clauses.push(clause);

        ClauseRef(self.clauses.len() as u32 - 1)
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
        let clause = LongClause::new(&lits);

        assert_eq!(lits.to_vec(), clause.iter().copied().collect::<Vec<_>>());
    }
}
