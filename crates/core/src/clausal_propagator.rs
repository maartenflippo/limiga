use crate::{
    assignment::Trail,
    clause::{ClauseDb, ClauseRef},
    lit::{Lit, Var},
    storage::KeyedVec,
};

#[derive(Default)]
pub struct ClausalPropagator {
    trail_idx: usize,
    watch_list: KeyedVec<Lit, Vec<ClauseRef>>,
}

impl ClausalPropagator {
    pub fn grow_to(&mut self, var: Var) {
        self.watch_list.grow_to(Lit::positive(var));
    }

    pub fn add_clause(&mut self, clause_ref: ClauseRef, clauses: &ClauseDb) {
        let clause = &clauses[clause_ref];

        self.watch_list[clause.head[0]].push(clause_ref);
        self.watch_list[clause.head[1]].push(clause_ref);
    }

    pub fn propagate(&mut self, trail: &mut Trail, db: &mut ClauseDb) -> Option<Conflict> {
        println!("c propagating...");

        for idx in self.trail_idx..trail.len() {
            let trail_lit = trail[idx];
            let false_lit = !trail_lit;
            println!("c  processing trail lit {trail_lit:?}");

            let constraints = std::mem::take(&mut self.watch_list[false_lit]);
            println!("c  watched constraints: {constraints:?}");

            for i in 0..constraints.len() {
                let clause_ref = constraints[i];

                if !self.propagate_clause(clause_ref, false_lit, trail, db) {
                    // The clause is conflicting, copy the remaining watches and return the
                    // appropriate conflict.

                    for j in i + 1..constraints.len() {
                        self.watch_list[false_lit].push(constraints[j]);
                    }

                    self.trail_idx = trail.len();

                    return Some(Conflict {
                        literal: trail_lit,
                        empty_clause: clause_ref,
                    });
                }
            }
        }

        self.trail_idx = trail.len();

        None
    }

    fn propagate_clause(
        &mut self,
        clause_ref: ClauseRef,
        false_lit: Lit,
        trail: &mut Trail,
        db: &mut ClauseDb,
    ) -> bool {
        let clause = &mut db[clause_ref];

        // Make sure the false literal is at position 1 in the clause.
        if clause.head[0] == false_lit {
            clause.head.swap(0, 1);
        }

        // If the 0th watch is true, then clause is already satisfied.
        if trail.value(clause.head[0]) == Some(true) {
            self.watch_list[false_lit].push(clause_ref);
            return true;
        }

        // Look for a new literal to watch.
        for tail_idx in 0..clause.tail.len() {
            let candidate = clause.tail[tail_idx];
            if trail.value(candidate) != Some(false) {
                clause.head[1] = candidate;
                clause.tail[tail_idx] = false_lit;

                self.watch_list[!clause.head[1]].push(clause_ref);
                return true;
            }
        }

        // The clause is unit under the current assignment.
        self.watch_list[false_lit].push(clause_ref);

        println!(
            "c  propagating {:?} because of {clause_ref:?}",
            clause.head[0]
        );

        trail.enqueue(clause.head[0], clause_ref)
    }
}

pub struct Conflict {
    pub literal: Lit,
    pub empty_clause: ClauseRef,
}
