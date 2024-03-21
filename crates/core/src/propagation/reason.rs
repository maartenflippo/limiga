use std::fmt::Debug;
use std::{borrow::Cow, ops::Deref};

use crate::{
    atom::Atom,
    clause::{ClauseDb, ClauseRef},
    lit::Lit,
};

#[derive(Debug, Default)]
pub enum Reason<Domains> {
    #[default]
    Decision,
    Clause(ClauseRef),
    Explanation {
        /// The literal which was propagated.
        propagated_lit: Lit,
        /// The explanation for the propagation.
        explanation: Explanation<Domains>,
    },
}

impl<Domains> Reason<Domains> {
    pub fn as_clause<'clauses>(
        &self,
        clauses: &'clauses ClauseDb,
        domains: &Domains,
    ) -> Cow<'clauses, [Lit]> {
        match self {
            Reason::Decision => Cow::Owned([].into()),
            Reason::Clause(clause_ref) => Cow::Borrowed(clauses[*clause_ref].lits()),
            Reason::Explanation {
                propagated_lit,
                explanation,
            } => {
                let mut clause = vec![*propagated_lit];
                clause.extend(explanation.iter().map(|atom| !atom.as_lit(domains)));

                Cow::Owned(clause)
            }
        }
    }
}

impl<Domains> From<ClauseRef> for Reason<Domains> {
    fn from(clause_ref: ClauseRef) -> Self {
        Reason::Clause(clause_ref)
    }
}

#[derive(Default)]
pub struct Explanation<Domains>(Box<[Box<dyn Atom<Domains>>]>);

impl<Domains> Explanation<Domains> {
    /// Get the number of atoms in the explanation.
    #[allow(clippy::len_without_is_empty, reason = "explanations cannot be empty")]
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<Domains> Clone for Explanation<Domains> {
    fn clone(&self) -> Self {
        let atoms = self.0.iter().map(|atom| atom.boxed_clone()).collect();
        Explanation(atoms)
    }
}

impl<Domains> Debug for Explanation<Domains> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        self.0[0].fmt_debug(f)?;
        write!(f, "]")?;
        Ok(())
    }
}

impl<Domains> FromIterator<Box<dyn Atom<Domains>>> for Explanation<Domains> {
    fn from_iter<T: IntoIterator<Item = Box<dyn Atom<Domains>>>>(iter: T) -> Self {
        let atoms = iter.into_iter().collect();
        Explanation(atoms)
    }
}

impl<Domains, LitSlice> From<LitSlice> for Explanation<Domains>
where
    LitSlice: Into<Box<[Box<dyn Atom<Domains>>]>>,
{
    fn from(value: LitSlice) -> Self {
        Explanation(value.into())
    }
}

impl<Domains> Explanation<Domains> {
    pub fn iter(&self) -> impl Iterator<Item = &dyn Atom<Domains>> + '_ {
        self.0.iter().map(|atom| atom.deref())
    }
}
