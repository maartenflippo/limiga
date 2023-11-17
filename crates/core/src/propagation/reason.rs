use crate::{clause::ClauseRef, lit::Lit};

#[derive(Debug, Default, PartialEq, Eq)]
pub enum Reason {
    #[default]
    Decision,
    Clause(ClauseRef),
    Explanation(Box<[Lit]>),
}

impl Reason {
    pub fn from_explanation(propagated_lit: Lit, explanation: Explanation) -> Reason {
        let mut lits = explanation.0.into_vec();

        // The explanation is stored as the clause. This means applying De-Morgan on !explanation.
        // This is why the propagated literal is pushed as the negated version.
        lits.push(!propagated_lit);

        let last_idx = lits.len() - 1;
        lits.swap(0, last_idx);

        lits.iter_mut().for_each(|lit| *lit = !*lit);

        Reason::Explanation(lits.into())
    }

    // pub fn to_vec(&self, clauses: &ClauseDb) -> Vec<Lit> {
    //     match self {
    //         Reason::Decision => vec![],
    //         Reason::Clause(clause_ref) => clauses[*clause_ref].iter().copied().collect(),
    //         Reason::Conjunction(lits) => lits.to_vec(),
    //     }
    // }

    // pub fn to_clause(&mut self, clauses: &mut ClauseDb) -> ClauseRef {
    //     match self {
    //         Reason::Decision => unreachable!(),
    //         Reason::Clause(clause_ref) => *clause_ref,
    //         Reason::Conjunction(lits) => {
    //             let clause_ref = clauses.add_explanation_clause(lits);
    //             *self = Reason::Clause(clause_ref);
    //             clause_ref
    //         }
    //     }
    // }
}

impl From<ClauseRef> for Reason {
    fn from(clause_ref: ClauseRef) -> Self {
        Reason::Clause(clause_ref)
    }
}

#[derive(Debug, Default, Eq)]
pub struct Explanation(Box<[Lit]>);

impl PartialEq for Explanation {
    fn eq(&self, other: &Self) -> bool {
        self.0.len() == other.0.len() && self.0.iter().all(|lit| other.0.contains(lit))
    }
}

impl<LitSlice> From<LitSlice> for Explanation
where
    LitSlice: Into<Box<[Lit]>>,
{
    fn from(value: LitSlice) -> Self {
        Explanation(value.into())
    }
}

impl Explanation {
    pub fn into_vec(self) -> Vec<Lit> {
        self.0.into_vec()
    }
}
