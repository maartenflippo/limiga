use crate::{
    search::{Branch, Partitioner},
    Variable,
};

pub struct DomainMin;

impl<Var, Store> Partitioner<Var, Store, 2> for DomainMin
where
    Var: Variable<Store> + Clone + 'static,
    Var::Value: 'static,
{
    fn partition(&mut self, variable: Var, store: &Store) -> [Branch<Store>; 2] {
        let var2 = variable.clone();

        let val1 = variable.min(store);
        let val2 = variable.min(store);

        [
            Box::new(move |s: &mut Store| {
                let val = val1;
                variable.fix(s, &val);
            }) as Branch<Store>,
            Box::new(move |store: &mut Store| {
                let val = val2;
                var2.remove(store, &val);
            }) as Branch<Store>,
        ]
    }
}
