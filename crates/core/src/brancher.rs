use crate::{
    assignment::Trail,
    lit::{Lit, Var},
};

pub trait Brancher {
    fn on_new_var(&mut self, var: Var);
    fn next_decision(&mut self, trail: &Trail) -> Option<Lit>;
}

#[derive(Default)]
pub struct NaiveBrancher {
    vars: Vec<Var>,
}

impl Brancher for NaiveBrancher {
    fn on_new_var(&mut self, var: Var) {
        self.vars.push(var);
    }

    fn next_decision(&mut self, trail: &Trail) -> Option<Lit> {
        self.vars
            .iter()
            .copied()
            .map(Lit::positive)
            .find(|&lit| trail.value(lit).is_none())
    }
}
