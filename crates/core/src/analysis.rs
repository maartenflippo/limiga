use log::trace;

use crate::{
    brancher::Brancher,
    clause::ClauseDb,
    domains::Conflict,
    implication_graph::ImplicationGraph,
    lit::{Lit, Var},
    propagation::Reason,
    search_tree::SearchTree,
    storage::KeyedVec,
    trail::Trail,
};

/// Responsible for clause learning when a conflict is encountered. The analyzer uses the 1-UIP
/// learning strategy.
#[derive(Default)]
pub struct ConflictAnalyzer {
    /// The working buffer on which analysis is performed.
    buffer: Vec<Lit>,
    /// The number of literals in the buffer that have been assigned at the level of the conflict
    /// (i.e. the current decision level).
    current_level_count: usize,
    /// For every variable, indicate whether it has been encountered during the analysis.
    seen: KeyedVec<Var, bool>,
    /// Variables which have been seen during conflict analysis.
    to_clear: Vec<Var>,
    /// The DFS stack for clause minimization.
    stack: Vec<Lit>,
}

/// The result of conflict analysis.
pub struct Analysis<'a> {
    /// The clause to learn.
    pub learned_clause: &'a [Lit],
    /// The decision level to backjump to.
    pub backjump_level: usize,
}

impl ConflictAnalyzer {
    pub fn grow_to(&mut self, var: Var) {
        self.seen.grow_to(var);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn analyze<Domains, SearchProc: Brancher>(
        &mut self,
        conflict: Conflict<Domains>,
        clauses: &ClauseDb,
        implication_graph: &ImplicationGraph<Domains>,
        search_tree: &SearchTree,
        trail: &Trail,
        brancher: &mut SearchProc,
        domains: &Domains,
    ) -> Analysis {
        self.current_level_count = 0;
        self.buffer.clear();

        let lits = conflict.lits(clauses, domains);

        trace!("conflict clause:    {:?}", lits);
        trace!(
            "   decision levels: {:?}",
            lits.iter()
                .map(|lit| search_tree.decision_level(lit.var()))
                .collect::<Vec<_>>()
        );

        // Add the literals of the current confl to the buffer. Care is taken to avoid adding
        // duplicate literals.
        for p in lits.iter() {
            self.add_literal(*p, search_tree, brancher);
        }

        // Resolve the clause in self.buffer until only one literal remains from the current
        // decision level.
        for lit in trail.iter().rev() {
            assert!(
                self.current_level_count > 0,
                "At least one literal has to have been assigned at the current decision level."
            );

            if !self.seen[lit.var()] {
                // The literal is not in the working clause, so there is nothing to do. Proceed to
                // the next literal.
                continue;
            }

            // We resolve on `lit`, which removes it from the working clause.
            self.current_level_count -= 1;

            if self.current_level_count == 0 {
                // We have reached the first UIP. The procedure terminates with the asserting
                // literal in the 0th spot of the learned clause.
                //
                // Note: `add_literal` does not actually add literals to self.buffer if they have
                // been assigned at the current decision level. Hence, lit is not in the buffer.
                self.buffer.push(!lit);

                // Ensure the asserting literal is at the beginning of the clause.
                let last_idx = self.buffer.len() - 1;
                self.buffer.swap(0, last_idx);

                break;
            }

            // Add the reason of the propagated literal to the clause. Since we are traversing the
            // trail in reverse order, and there is at least one more literal on the trail that was
            // assigned at the current decision level, the current literal *MUST* have been
            // propagated.
            let reason = implication_graph
                .reason(lit.var())
                .as_clause(clauses, domains);

            assert_eq!(
                lit, reason[0],
                "the propagated literal is at the first position in the clause"
            );

            for p in reason.iter().skip(1) {
                self.add_literal(*p, search_tree, brancher);
            }
        }

        self.minimize_clause(clauses, implication_graph, search_tree, domains);

        // Reset the seen state for any variable we encountered during analysis.
        for var in self.to_clear.drain(..) {
            self.seen[var] = false;
        }

        let (idx, backjump_level) = self
            .buffer
            .iter()
            .enumerate()
            .skip(1)
            .map(|(idx, lit)| (idx, search_tree.decision_level(lit.var())))
            .max_by_key(|(_, dl)| *dl)
            .unwrap_or((0, 0));

        if idx > 1 {
            // Ensure the literal with the highest decision level is at index 1 of the learned clause.
            self.buffer.swap(1, idx);
        }

        trace!("backtracing to {backjump_level}");

        Analysis {
            learned_clause: &self.buffer,
            backjump_level,
        }
    }

    fn add_literal<SearchProc: Brancher>(
        &mut self,
        lit: Lit,
        search_tree: &SearchTree,
        brancher: &mut SearchProc,
    ) {
        if search_tree.decision_level(lit.var()) > 0 && !self.seen[lit.var()] {
            self.seen[lit.var()] = true;
            self.to_clear.push(lit.var());
            brancher.on_variable_activated(lit.var());

            if search_tree.decision_level(lit.var()) == search_tree.depth() {
                self.current_level_count += 1;
            } else {
                self.buffer.push(lit);
            }
        }
    }

    fn minimize_clause<Domains>(
        &mut self,
        clauses: &ClauseDb,
        implication_graph: &ImplicationGraph<Domains>,
        search_tree: &SearchTree,
        domains: &Domains,
    ) {
        // we always keep the first literal
        let mut idx = 0;

        'next_lit: while idx + 1 < self.buffer.len() {
            idx += 1;

            let lit = self.buffer[idx];

            if matches!(implication_graph.reason(lit.var()), Reason::Decision) {
                continue;
            }

            // Start the DFS
            self.stack.clear();
            self.stack.push(!lit);

            // Used to remember which var_flags are set during this DFS
            let top = self.to_clear.len();

            while let Some(lit) = self.stack.pop() {
                let reason_lits = implication_graph
                    .reason(lit.var())
                    .as_clause(clauses, domains);

                for &reason_lit in reason_lits.iter() {
                    let reason_level = search_tree.decision_level(reason_lit.var());

                    if !self.seen[reason_lit.var()] && reason_level > 0 {
                        // We haven't established reason_lit to be redundant, haven't visited it yet and
                        // it's not implied by unit clauses.

                        if matches!(implication_graph.reason(reason_lit.var()), Reason::Decision) {
                            // reason_lit is a decision not in the clause
                            // Reset the var_flags set during _this_ DFS.
                            for var in self.to_clear.drain(top..) {
                                self.seen[var] = false;
                            }
                            continue 'next_lit;
                        } else {
                            self.seen[reason_lit.var()] = true;
                            self.to_clear.push(reason_lit.var());
                            self.stack.push(!reason_lit);
                        }
                    }
                }
            }

            self.buffer.swap_remove(idx);
        }
    }
}
