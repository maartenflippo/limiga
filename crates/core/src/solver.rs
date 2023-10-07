use bitvec::vec::BitVec;

use crate::{
    assignment::Trail,
    brancher::Brancher,
    clausal_propagator::{ClausalPropagator, Conflict},
    clause::{ClauseDb, ClauseRef},
    lit::{Lit, Var},
};

#[derive(Default)]
pub struct Solver<B> {
    clauses: ClauseDb,
    trail: Trail,
    learned_clause_buffer: Vec<Lit>,
    decision_level: usize,
    brancher: B,
    propagator: ClausalPropagator,
    seen: BitVec,

    next_var_code: u32,
}

impl<B: Brancher> Solver<B> {
    pub fn new_lits(&mut self) -> impl Iterator<Item = Lit> + '_ {
        NewLitIterator { solver: self }
    }

    pub fn add_clause(&mut self, lits: impl AsRef<[Lit]>) {
        let lits = lits.as_ref();

        assert!(!lits.is_empty(), "Cannot explicitly add the empty clause.");

        println!("c adding clause {lits:?}");

        if lits.len() == 1 {
            self.trail.enqueue(lits[0], ClauseRef::default());
        } else {
            let clause_ref = self.clauses.add_clause(lits.as_ref());
            self.propagator.add_clause(clause_ref, &self.clauses);
            println!("c  with id {clause_ref:?}");
        }
    }

    pub fn solve(&mut self) -> SolveResult<'_, B> {
        loop {
            match self
                .propagator
                .propagate(&mut self.trail, &mut self.clauses)
            {
                Some(conflict) => {
                    println!("c conflict...");
                    println!("c  empty clause = {:?}", conflict.empty_clause);

                    if self.decision_level == 0 {
                        return SolveResult::Unsatisfiable;
                    }

                    let backjump_level = self.analyze(conflict);
                    println!("c  learned clause = {:?}", self.learned_clause_buffer);
                    println!("c  backtracking to dl {}", backjump_level);

                    let clause_ref = self.clauses.add_clause(&self.learned_clause_buffer);
                    self.backtrack_to(backjump_level);
                    self.trail
                        .enqueue(self.learned_clause_buffer[0], clause_ref);
                }

                None => {
                    self.trail.new_decision_level();
                    self.decision_level += 1;

                    if let Some(decision) = self.brancher.next_decision(&self.trail) {
                        println!("c decide {decision:?}");
                        self.trail.enqueue(decision, ClauseRef::default());
                    } else {
                        return SolveResult::Satisfiable(Solution { solver: self });
                    }
                }
            }
        }
    }

    fn analyze(&mut self, conflict: Conflict) -> usize {
        self.learned_clause_buffer.clear();
        self.seen.iter_mut().for_each(|mut v| *v = false);

        let mut p = None;
        let mut confl = conflict.empty_clause;

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
                let q = p_reason[j];

                if !self.seen[q.var().code() as usize] {
                    self.seen.set(q.var().code() as usize, true);
                    if self.trail.get_decision_level(q) == self.decision_level {
                        counter += 1;
                    } else if self.trail.get_decision_level(q) > 0 {
                        self.learned_clause_buffer.push(!q);
                        backtrack_to = usize::max(backtrack_to, self.trail.get_decision_level(q));
                    }
                }
            }

            loop {
                p = self.trail.last();
                confl = self.trail.get_reason(p.unwrap());
                self.trail.pop();

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

        backtrack_to
    }

    fn backtrack_to(&mut self, decision_level: usize) {
        self.trail.backtrack_to(decision_level);
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
        self.solver.trail.value(Lit::positive(var)).unwrap()
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

        self.solver.trail.grow_to(var);
        self.solver.propagator.grow_to(var);
        self.solver.brancher.on_new_var(var);

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
