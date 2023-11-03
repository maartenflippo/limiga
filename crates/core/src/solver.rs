use log::trace;

use crate::{
    analysis::ConflictAnalyzer,
    assignment::Assignment,
    brancher::Brancher,
    clause::{ClauseDb, ClauseRef},
    implication_graph::ImplicationGraph,
    lit::{Lit, Var},
    preprocessor::{ClausePreProcessor, PreProcessedClause},
    search_tree::SearchTree,
    storage::KeyedVec,
    termination::Terminator,
    trail::Trail,
};

pub struct Solver<SearchProc, Timer> {
    brancher: SearchProc,
    timer: Timer,

    preprocessor: ClausePreProcessor,
    analyzer: ConflictAnalyzer,
    clauses: ClauseDb,
    implication_graph: ImplicationGraph,
    search_tree: SearchTree,
    state: State,

    trail: Trail,
    assignment: Assignment,

    next_propagation_idx: usize,
    watch_list: KeyedVec<Lit, Vec<ClauseRef>>,
    next_var_code: u32,
}

#[derive(Default, PartialEq, Eq)]
enum State {
    #[default]
    Consistent,
    ConflictAtRoot,
}

impl<SearchProc, Timer> Solver<SearchProc, Timer> {
    pub fn new(brancher: SearchProc, timer: Timer) -> Self {
        Solver {
            brancher,
            timer,
            clauses: Default::default(),
            search_tree: Default::default(),
            state: Default::default(),
            trail: Default::default(),
            assignment: Default::default(),
            next_propagation_idx: 0,
            watch_list: Default::default(),
            next_var_code: 0,
            preprocessor: Default::default(),
            analyzer: Default::default(),
            implication_graph: Default::default(),
        }
    }
}

impl<SearchProc, Timer> Solver<SearchProc, Timer>
where
    SearchProc: Brancher,
{
    pub fn add_clause(&mut self, lits: impl IntoIterator<Item = Lit>) {
        if self.state == State::ConflictAtRoot {
            return;
        }

        let root_assignment = {
            let lits = match self.preprocessor.preprocess(lits, &self.assignment) {
                PreProcessedClause::Satisfiable => return,
                PreProcessedClause::Lits(lits) => lits,
            };

            if lits.is_empty() {
                self.state = State::ConflictAtRoot;
                return;
            }

            if lits.len() > 1 {
                let clause_ref = self.clauses.add_clause(lits);
                trace!("adding clause {lits:?} with id {clause_ref:?}");

                let clause = &self.clauses[clause_ref];
                self.watch_list[clause.head[0]].push(clause_ref);
                self.watch_list[clause.head[1]].push(clause_ref);
                return;
            }

            lits[0]
        };

        if !self.enqueue(root_assignment, ClauseRef::default()) {
            self.state = State::ConflictAtRoot;
        }

        trace!("adding clause [{root_assignment:?}] as assignment");
    }

    pub fn new_lits(&mut self) -> impl Iterator<Item = Lit> + '_ {
        NewLitIterator {
            solver: self,
            has_introduced_new_literal: false,
        }
    }

    fn enqueue(&mut self, lit: Lit, reason: ClauseRef) -> bool {
        if let Some(false) = self.assignment.value(lit) {
            return false;
        }

        self.trail.enqueue(lit);
        self.assignment.assign(lit);
        self.implication_graph.add(lit.var(), reason);
        self.search_tree.register_assignment(lit);

        if reason != ClauseRef::default() {
            assert_eq!(
                lit, self.clauses[reason][0],
                "Propagated literals should be the first literal in the clause."
            );
        }

        true
    }

    fn backtrack_to(&mut self, decision_level: usize) {
        self.trail.backtrack_to(decision_level).for_each(|lit| {
            self.assignment.unassign(lit);
            self.brancher.on_variable_unassigned(lit.var());
        });

        self.search_tree.cut(decision_level);
        self.next_propagation_idx = self.trail.len();
    }

    fn propagate(&mut self) -> Option<ClauseRef> {
        trace!("propagating...");
        while self.next_propagation_idx < self.trail.len() {
            let trail_lit = self.trail[self.next_propagation_idx];
            let false_lit = !trail_lit;
            self.next_propagation_idx += 1;

            trace!("processing {trail_lit:?}");

            let constraints = std::mem::take(&mut self.watch_list[false_lit]);

            trace!("watched constraints {constraints:?}");

            for i in 0..constraints.len() {
                let clause_ref = constraints[i];

                if !self.propagate_clause(clause_ref, false_lit) {
                    // The clause is conflicting, copy the remaining watches and return the
                    // appropriate conflict.

                    for &constraint in constraints.iter().skip(i + 1) {
                        trace!("adding {constraint:?} to the watch list of {false_lit:?}");
                        self.watch_list[false_lit].push(constraint);
                    }

                    self.next_propagation_idx = self.trail.len();

                    return Some(clause_ref);
                }
            }
        }

        None
    }

    fn propagate_clause(&mut self, clause_ref: ClauseRef, false_lit: Lit) -> bool {
        let lit_to_propagate = {
            let clause = &mut self.clauses[clause_ref];
            trace!("propagating clause {clause:?}");

            // Make sure the false literal is at position 1 in the clause.
            if clause.head[0] == false_lit {
                clause.head.swap(0, 1);
            }

            // If the 0th watch is true, then clause is already satisfied.
            if self.assignment.value(clause.head[0]) == Some(true) {
                trace!("clause is satisfied because of 0th literal");
                self.watch_list[false_lit].push(clause_ref);
                return true;
            }

            // Look for a new literal to watch.
            for tail_idx in 0..clause.tail.len() {
                let candidate = clause.tail[tail_idx];
                if self.assignment.value(candidate) != Some(false) {
                    trace!("found new watch literal {candidate:?}");
                    clause.head[1] = candidate;
                    clause.tail[tail_idx] = false_lit;

                    self.watch_list[clause.head[1]].push(clause_ref);
                    return true;
                }
            }

            // The clause is unit under the current assignment.
            self.watch_list[false_lit].push(clause_ref);
            clause.head[0]
        };

        trace!(
            "propagating {:?} because of {clause_ref:?}",
            lit_to_propagate
        );

        self.enqueue(lit_to_propagate, clause_ref)
    }
}

impl<SearchProc, Timer> Solver<SearchProc, Timer>
where
    SearchProc: Brancher,
    Timer: Terminator,
{
    pub fn solve(&mut self) -> SolveResult<'_> {
        if self.state == State::ConflictAtRoot {
            return SolveResult::Unsatisfiable;
        }

        while !self.timer.should_stop() {
            match self.propagate() {
                Some(conflict) => {
                    trace!("conflict at dl {}", self.search_tree.depth());

                    if self.search_tree.is_at_root() {
                        return SolveResult::Unsatisfiable;
                    }

                    let (literal_to_enqueue, reason, backjump_level) = {
                        let analysis = self.analyzer.analyze(
                            conflict,
                            &self.clauses,
                            &self.implication_graph,
                            &self.search_tree,
                            &self.trail,
                            &mut self.brancher,
                        );

                        let clause_ref = if analysis.learned_clause.len() > 1 {
                            self.clauses.add_clause(analysis.learned_clause)
                        } else {
                            ClauseRef::default()
                        };

                        (
                            analysis.learned_clause[0],
                            clause_ref,
                            analysis.backjump_level,
                        )
                    };

                    self.backtrack_to(backjump_level);

                    assert!(
                        self.enqueue(literal_to_enqueue, reason),
                        "conflicting asserting literal"
                    );

                    self.brancher.on_conflict();
                }

                None => {
                    self.trail.push();
                    self.search_tree.branch();

                    if let Some(decision) = self.brancher.next_decision(&self.assignment) {
                        trace!("decided {decision:?}");
                        assert!(
                            self.enqueue(decision, ClauseRef::default()),
                            "decided already assigned literal"
                        );
                    } else {
                        return SolveResult::Satisfiable(Solution {
                            assignment: &mut self.assignment,
                            next_new_var_code: self.next_var_code,
                        });
                    }
                }
            }
        }

        SolveResult::Unknown
    }
}

pub enum SolveResult<'solver> {
    /// A solution has been found for the formula.
    Satisfiable(Solution<'solver>),
    /// No solution exists for the formula.
    Unsatisfiable,
    /// The solver was interrupted before reaching a conclusion.
    Unknown,
}

pub struct Solution<'assignment> {
    assignment: &'assignment mut Assignment,
    next_new_var_code: u32,
}

impl Solution<'_> {
    pub fn value(&self, var: Var) -> bool {
        self.assignment.value(Lit::positive(var)).unwrap()
    }

    pub fn vars(&self) -> impl Iterator<Item = Var> + '_ {
        (0..self.next_new_var_code).map(|code| Var::try_from(code).unwrap())
    }
}

struct NewLitIterator<'a, SearchProc, Timer> {
    solver: &'a mut Solver<SearchProc, Timer>,
    has_introduced_new_literal: bool,
}

impl<SearchProc, Timer> Iterator for NewLitIterator<'_, SearchProc, Timer>
where
    SearchProc: Brancher,
{
    type Item = Lit;

    fn next(&mut self) -> Option<Self::Item> {
        let var = Var::try_from(self.solver.next_var_code).expect("valid var code");
        let lit = Lit::positive(var);
        self.solver.brancher.on_new_var(var);

        self.solver.next_var_code += 1;
        self.has_introduced_new_literal = true;

        Some(lit)
    }
}

impl<SearchProc, Timer> Drop for NewLitIterator<'_, SearchProc, Timer> {
    fn drop(&mut self) {
        if self.has_introduced_new_literal {
            let last_var = Var::try_from(self.solver.next_var_code - 1)
                .expect("was created successfully previously as well");

            self.solver.assignment.grow_to(last_var);
            self.solver.implication_graph.grow_to(last_var);
            self.solver.search_tree.grow_to(last_var);
            self.solver.watch_list.grow_to(Lit::positive(last_var));
            self.solver.analyzer.grow_to(last_var);
        }
    }
}
