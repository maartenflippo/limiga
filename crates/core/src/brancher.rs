use crate::{
    assignment::Assignment,
    lit::{Lit, Var},
    storage::KeyedVec,
};

pub trait Brancher {
    /// Called when a new variable is created in the solver.
    fn on_new_var(&mut self, var: Var);

    /// Called when the given variable is seen during conflict analysis. The variable in question
    /// is guaranteed to have been passed to [`on_new_var()`] before this is called.
    fn on_variable_activated(&mut self, var: Var);

    /// Called when the solver finishes conflict analysis.
    fn on_conflict(&mut self);

    /// Add a variable back into contention if it had previously been assigned.
    fn on_variable_unassigned(&mut self, var: Var);

    /// Provide the solver with the next decision. The returned literal should be unassigned under
    /// the given assignment.
    fn next_decision(&mut self, assignment: &Assignment) -> Option<Lit>;
}

pub struct VsidsBrancher {
    /// The activity of each variable.
    activities: KeyedVec<Var, f64>,
    /// A binary heap of the variables.
    heap: Vec<Var>,
    /// The position in the binary heap for each variable.
    position: KeyedVec<Var, Option<usize>>,

    activity_increment: f64,
    decay: f64,
}

impl VsidsBrancher {
    pub fn new(decay: f64) -> Self {
        VsidsBrancher {
            activities: Default::default(),
            heap: Default::default(),
            position: Default::default(),
            activity_increment: 1.0,
            decay,
        }
    }

    fn rescale_activities(&mut self) {
        self.activities
            .iter_mut()
            .for_each(|activity| *activity *= 1e-100);
    }

    /// Move a variable closer to the root until the heap property is satisfied.
    fn sift_up(&mut self, mut pos: usize) {
        let var = self.heap[pos];
        loop {
            if pos == 0 {
                return;
            }
            let parent_pos = (pos - 1) / 2;
            let parent_var = self.heap[parent_pos];
            if self.activities[parent_var] >= self.activities[var] {
                return;
            }
            self.position[var] = Some(parent_pos);
            self.heap[parent_pos] = var;
            self.position[parent_var] = Some(pos);
            self.heap[pos] = parent_var;
            pos = parent_pos;
        }
    }

    /// Move a variable away from the root until the heap property is satisfied.
    fn sift_down(&mut self, mut pos: usize) {
        let var = self.heap[pos];
        loop {
            let mut largest_pos = pos;
            let mut largest_var = var;

            let left_pos = pos * 2 + 1;
            if left_pos < self.heap.len() {
                let left_var = self.heap[left_pos];

                if self.activities[largest_var] < self.activities[left_var] {
                    largest_pos = left_pos;
                    largest_var = left_var;
                }
            }

            let right_pos = pos * 2 + 2;
            if right_pos < self.heap.len() {
                let right_var = self.heap[right_pos];

                if self.activities[largest_var] < self.activities[right_var] {
                    largest_pos = right_pos;
                    largest_var = right_var;
                }
            }

            if largest_pos == pos {
                return;
            }

            self.position[var] = Some(largest_pos);
            self.heap[largest_pos] = var;
            self.position[largest_var] = Some(pos);
            self.heap[pos] = largest_var;
            pos = largest_pos;
        }
    }
}

impl Brancher for VsidsBrancher {
    fn on_new_var(&mut self, var: Var) {
        self.activities.grow_to(var);
        self.position.grow_to(var);

        self.on_variable_unassigned(var);
    }

    fn on_variable_activated(&mut self, var: Var) {
        let activity = &mut self.activities[var];
        *activity += self.activity_increment;

        if *activity > 1e100 {
            self.rescale_activities();
        }

        if let Some(pos) = self.position[var] {
            self.sift_up(pos);
        }
    }

    fn on_conflict(&mut self) {
        self.activity_increment *= self.decay;
    }

    fn on_variable_unassigned(&mut self, var: Var) {
        if self.position[var].is_none() {
            let position = self.heap.len();
            self.position[var] = Some(position);
            self.heap.push(var);
            self.sift_up(position);
        }
    }

    fn next_decision(&mut self, assignment: &Assignment) -> Option<Lit> {
        while !self.heap.is_empty() {
            let var = self.heap.swap_remove(0);
            if !self.heap.is_empty() {
                let top_var = self.heap[0];
                self.position[top_var] = Some(0);
                self.sift_down(0);
            }
            self.position[var] = None;

            let lit = Lit::positive(var);
            if assignment.is_unassigned(lit) {
                return Some(lit);
            }
        }

        None
    }
}
