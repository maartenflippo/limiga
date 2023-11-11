use crate::lit::Lit;

pub struct Reason(Box<[Lit]>);

impl Reason {
    /// Create a reason based on a propositional conjunction (i.e. a set of literals).
    pub fn from_literals(lits: &[Lit]) -> Self {
        Reason(lits.into())
    }
}
