use bitvec::vec::BitVec;
use log::trace;

use crate::{
    assignment::Assignment,
    brancher::Brancher,
    clause::{ClauseDb, ClauseRef},
    lit::{Lit, Var},
    storage::KeyedVec,
    trail::Trail,
};

#[derive(Default)]
pub struct Solver<B> {
    brancher: B,

    clauses: ClauseDb,
    learned_clause_buffer: Vec<Lit>,
    decision_level: usize,
    seen: BitVec,
    state: State,

    trail: Trail,
    assignment: Assignment,
    reasons: KeyedVec<Lit, ClauseRef>,
    decided_at: KeyedVec<Lit, usize>,

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

impl<B: Brancher> Solver<B> {
    pub fn new_lits(&mut self) -> impl Iterator<Item = Lit> + '_ {
        NewLitIterator { solver: self }
    }

    pub fn add_clause(&mut self, lits: impl AsRef<[Lit]>) {
        if self.state == State::ConflictAtRoot {
            return;
        }

        let lits = lits.as_ref();

        if lits.is_empty() {
            self.state = State::ConflictAtRoot;
            return;
        }

        if lits.len() == 1 {
            if !self.enqueue(lits[0], ClauseRef::default()) {
                self.state = State::ConflictAtRoot;
            }

            trace!("adding clause {lits:?} as assignment");
        } else {
            let clause_ref = self.clauses.add_clause(lits);
            trace!("adding clause {lits:?} with id {clause_ref:?}");

            let clause = &self.clauses[clause_ref];
            self.watch_list[clause.head[0]].push(clause_ref);
            self.watch_list[clause.head[1]].push(clause_ref);
        }
    }

    pub fn solve(&mut self) -> SolveResult<'_, B> {
        if self.state == State::ConflictAtRoot {
            return SolveResult::Unsatisfiable;
        }

        loop {
            match self.propagate() {
                Some(conflict) => {
                    trace!("conflict at dl {}", self.decision_level);

                    if self.decision_level == 0 {
                        return SolveResult::Unsatisfiable;
                    }

                    let backjump_level = self.analyze(conflict);

                    self.backtrack_to(backjump_level);

                    let clause_ref = if self.learned_clause_buffer.len() > 1 {
                        self.clauses.add_clause(&self.learned_clause_buffer)
                    } else {
                        ClauseRef::default()
                    };

                    assert!(
                        self.enqueue(self.learned_clause_buffer[0], clause_ref),
                        "conflicting asserting literal"
                    );
                }

                None => {
                    self.trail.push();
                    self.decision_level += 1;

                    if let Some(decision) = self.brancher.next_decision(&self.assignment) {
                        trace!("decided {decision:?}");
                        assert!(
                            self.enqueue(decision, ClauseRef::default()),
                            "decided already assigned literal"
                        );
                    } else {
                        return SolveResult::Satisfiable(Solution { solver: self });
                    }
                }
            }
        }
    }

    fn enqueue(&mut self, lit: Lit, reason: ClauseRef) -> bool {
        if let Some(false) = self.assignment.value(lit) {
            return false;
        }

        self.trail.enqueue(lit);
        self.assignment.assign(lit);
        self.reasons[lit] = reason;
        self.decided_at[lit] = self.decision_level;

        true
    }

    fn analyze(&mut self, empty_clause: ClauseRef) -> usize {
        trace!("analyzing...");

        self.learned_clause_buffer.clear();
        self.seen.iter_mut().for_each(|mut v| *v = false);

        let mut p = None;
        let mut confl = empty_clause;

        // Leave space for the asserting literal.
        self.learned_clause_buffer
            .push(Lit::positive(unsafe { Var::new_unchecked(0) }));

        let mut backtrack_to = 0;
        let mut counter = 0;

        loop {
            let p_reason = &self.clauses[confl];
            let start_idx = if let Some(p) = p {
                assert_eq!(p, p_reason[0]);
                1
            } else {
                0
            };

            for j in start_idx..p_reason.len() {
                let q = !p_reason[j];
                let q_decision_level = self.decided_at[q];

                if !self.seen[q.var().code() as usize] {
                    self.seen.set(q.var().code() as usize, true);
                    if self.decided_at[q] == self.decision_level {
                        counter += 1;
                    } else if q_decision_level > 0 {
                        self.learned_clause_buffer.push(!q);
                        backtrack_to = usize::max(backtrack_to, q_decision_level);
                    }
                }
            }

            loop {
                p = Some(self.trail[self.trail.len() - 1]);
                confl = self.reasons[p.unwrap()];

                self.undo_one();

                if self.seen[p.unwrap().var().code() as usize] {
                    break;
                }
            }

            counter -= 1;

            if counter == 0 {
                break;
            }
        }

        self.learned_clause_buffer[0] = !(p.unwrap());

        trace!("learned clause = {:?}", self.learned_clause_buffer);

        backtrack_to
    }

    fn backtrack_to(&mut self, decision_level: usize) {
        self.trail.backtrack_to(decision_level).for_each(|lit| {
            self.assignment.unassign(lit);
        });

        self.decision_level = decision_level;
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

    fn undo_one(&mut self) {
        if let Some(lit) = self.trail.pop() {
            self.assignment.unassign(lit);
        }
    }
}

pub enum SolveResult<'solver, B> {
    Satisfiable(Solution<'solver, B>),
    Unsatisfiable,
}

pub struct Solution<'solver, B> {
    solver: &'solver mut Solver<B>,
}

impl<B> Solution<'_, B> {
    pub fn value(&self, var: Var) -> bool {
        self.solver.assignment.value(Lit::positive(var)).unwrap()
    }

    pub fn vars(&self) -> impl Iterator<Item = Var> + '_ {
        (0..self.solver.next_var_code).map(|code| Var::try_from(code).unwrap())
    }
}

struct NewLitIterator<'a, B> {
    solver: &'a mut Solver<B>,
}

impl<B: Brancher> Iterator for NewLitIterator<'_, B> {
    type Item = Lit;

    fn next(&mut self) -> Option<Self::Item> {
        let var = Var::try_from(self.solver.next_var_code).expect("valid var code");
        let lit = Lit::positive(var);

        self.solver.brancher.on_new_var(var);
        self.solver.assignment.grow_to(var);
        self.solver.reasons.grow_to(Lit::positive(var));
        self.solver.decided_at.grow_to(Lit::positive(var));
        self.solver.watch_list.grow_to(Lit::positive(var));

        self.solver.next_var_code += 1;

        Some(lit)
    }
}

impl<B> Drop for NewLitIterator<'_, B> {
    fn drop(&mut self) {
        self.solver
            .seen
            .resize_with(self.solver.next_var_code as usize, |_| false);
    }
}
