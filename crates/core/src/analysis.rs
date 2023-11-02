use bitvec::vec::BitVec;
use log::trace;

use crate::{
    clause::{ClauseDb, ClauseRef},
    implication_graph::ImplicationGraph,
    lit::{Lit, Var},
    search_tree::SearchTree,
};

/// Responsible for clause learning when a conflict is encountered. The analyzer uses the 1-UIP
/// learning strategy.
#[derive(Default)]
pub struct ConflictAnalyzer {
    /// The working buffer on which analysis is performed.
    buffer: Vec<Lit>,
    /// For every variable, indicate whether it has been encountered during the analysis.
    seen: BitVec,
}

/// The result of conflict analysis.
pub struct Analysis<'a> {
    /// The clause to learn.
    pub learned_clause: &'a [Lit],
    /// The decision level to backjump to.
    pub backjump_level: usize,
}

pub trait SolverState {
    /// Get the last literal from the trail. The invariants of conflict analysis are such
    /// that this literal always exists when asked for. Hence, if it doesn't, this may panic.
    fn pop_trail(&mut self) -> Lit;

    /// Called when a literal is encountered during the analysis procedure.
    fn on_literal_activated(&mut self, lit: Lit);
}


impl ConflictAnalyzer {
    pub fn grow_to(&mut self, var: Var) {
        self.seen.resize(var.code() as usize + 1, false);
    }

    pub fn analyze<State: SolverState>(
        &mut self,
        empty_clause: ClauseRef,
        clauses: &ClauseDb,
        implication_graph: &ImplicationGraph,
        search_tree: &SearchTree,
        state: &mut State,
    ) -> Analysis {
        trace!("analyzing...");

        self.buffer.clear();
        self.seen.iter_mut().for_each(|mut v| *v = false);

        let mut p = None;
        let mut confl = empty_clause;

        // Leave space for the asserting literal.
        self.buffer
            .push(Lit::positive(unsafe { Var::new_unchecked(0) }));

        let mut backtrack_to = 0;
        let mut counter = 0;

        loop {
            let p_reason = &clauses[confl];
            let start_idx = if let Some(p) = p {
                assert_eq!(p, p_reason[0]);
                1
            } else {
                0
            };

            for j in start_idx..p_reason.len() {
                let q = !p_reason[j];
                let q_decision_level = search_tree.decision_level(q.var());

                if !self.seen[q.var().code() as usize] {
                    state.on_literal_activated(q);
                    self.seen.set(q.var().code() as usize, true);

                    if q_decision_level == search_tree.depth() {
                        counter += 1;
                    } else if q_decision_level > 0 {
                        self.buffer.push(!q);
                        backtrack_to = usize::max(backtrack_to, q_decision_level);
                    }
                }
            }

            loop {
                let lit = state.pop_trail();
                confl = implication_graph.reason(lit);
                p = Some(lit);

                if self.seen[p.unwrap().var().code() as usize] {
                    break;
                }
            }

            counter -= 1;

            if counter == 0 {
                break;
            }
        }

        self.buffer[0] = !(p.unwrap());

        trace!("learned clause = {:?}", self.buffer);

        Analysis {
            learned_clause: &self.buffer,
            backjump_level: backtrack_to,
        }
    }
}
