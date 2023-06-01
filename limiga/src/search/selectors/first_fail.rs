use crate::{search::Selector, Variable};

pub struct FirstFail<Var> {
    vars: Vec<Var>,
}

impl<Var> FirstFail<Var> {
    pub fn new(vars: Vec<Var>) -> Self {
        FirstFail { vars }
    }
}

impl<Var, Store> Selector<Var, Store> for FirstFail<Var>
where
    Var: Variable<Store> + Clone,
{
    fn select(&mut self, store: &Store) -> Option<Var> {
        self.vars
            .iter()
            .filter(|var| var.size(store) > 1)
            .min_by_key(|var| var.size(store))
            .cloned()
    }
}
