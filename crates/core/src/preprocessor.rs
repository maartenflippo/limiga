use log::trace;

use crate::{assignment::Assignment, lit::Lit};

/// Performs pre-processing on clauses that are added to the solver.
#[derive(Default)]
pub struct ClausePreProcessor {
    /// The buffer on which preprocessing operates.
    buffer: Vec<Lit>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PreProcessedClause<'a> {
    /// The clause is already satisfied. Either because it contained a literal already assigned to
    /// true, or because it contained literals of opposite polarity.
    Satisfiable,
    /// The remaining literals after preprocessing. Note: there might be 0 literals remaining, in
    /// which case the problem is unsatisfiable.
    Lits(&'a [Lit]),
}

impl ClausePreProcessor {
    pub fn preprocess(
        &mut self,
        lits: impl IntoIterator<Item = Lit>,
        assignment: &Assignment,
    ) -> PreProcessedClause<'_> {
        self.buffer.clear();
        self.buffer.extend(lits);

        let original_len = self.buffer.len();

        self.buffer.sort();
        self.buffer.dedup();

        for i in 0..self.buffer.len() {
            let x = self.buffer[i];

            if assignment.value(x) == Some(true) {
                trace!("preprocessing concluded trivially satisfiable",);
                return PreProcessedClause::Satisfiable;
            }

            if i < self.buffer.len() - 1 {
                // Due to sorting, literals of opposite polarity are guaranteed to be next to each
                // other.
                let y = self.buffer[i + 1];
                if x == !y {
                    trace!("preprocessing concluded trivially satisfiable",);
                    return PreProcessedClause::Satisfiable;
                }
            }
        }

        trace!(
            "preprocessing removed {} lits",
            original_len - self.buffer.len()
        );

        PreProcessedClause::Lits(&self.buffer)
    }
}

#[cfg(test)]
mod tests {
    use crate::lit;

    use super::*;

    #[test]
    fn excluded_middle_preprocessed_to_true() {
        let mut preprocessor = ClausePreProcessor::default();
        let mut assignment = Assignment::default();
        assignment.grow_to(unsafe { lit!(1).var() });

        let result = preprocessor.preprocess(unsafe { [lit!(1), lit!(-1)] }, &assignment);

        assert_eq!(PreProcessedClause::Satisfiable, result);
    }

    #[test]
    fn duplicate_literals_are_removed() {
        let mut preprocessor = ClausePreProcessor::default();
        let mut assignment = Assignment::default();
        assignment.grow_to(unsafe { lit!(3).var() });

        let result =
            preprocessor.preprocess(unsafe { [lit!(1), lit!(2), lit!(3), lit!(1)] }, &assignment);

        assert_eq!(
            PreProcessedClause::Lits(unsafe { &[lit!(1), lit!(2), lit!(3)] }),
            result
        );
    }

    #[test]
    fn clauses_with_a_true_literal() {
        let mut preprocessor = ClausePreProcessor::default();
        let mut assignment = Assignment::default();
        assignment.grow_to(unsafe { lit!(3).var() });
        assignment.assign(unsafe { lit!(-2) });

        let result = preprocessor.preprocess(unsafe { [lit!(1), lit!(-2), lit!(3)] }, &assignment);

        assert_eq!(PreProcessedClause::Satisfiable, result);
    }
}
