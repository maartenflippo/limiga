pub mod partitioners;
pub mod selectors;

use std::marker::PhantomData;

/// A partition of the domain of the CSP. Created by the [`Brancher`] during search.
pub type Branch<Store> = Box<dyn FnOnce(&mut Store)>;

/// A brancher creates the nodes in the search tree.
pub trait Brancher<Store> {
    type Branches: IntoIterator<Item = Branch<Store>>;

    /// Create the new branches of the search tree. If all variables are fixed, and search is
    /// therefore completed, this should return [`None`].
    fn branch(&mut self, store: &Store) -> Option<Self::Branches>;
}

impl<Func, Branches, Store> Brancher<Store> for Func
where
    Branches: IntoIterator<Item = Branch<Store>>,
    Func: Fn(&Store) -> Option<Branches>,
{
    type Branches = Branches;

    fn branch(&mut self, store: &Store) -> Option<Self::Branches> {
        self(store)
    }
}

pub struct Search<Var, S, P, const BRANCHING_FACTOR: usize> {
    selector: S,
    partitioner: P,
    variable: PhantomData<Var>,
}

impl<Var, S, P, const BRANCHING_FACTOR: usize> Search<Var, S, P, BRANCHING_FACTOR> {
    pub fn new(selector: S, partitioner: P) -> Self {
        Search {
            selector,
            partitioner,
            variable: PhantomData,
        }
    }
}

pub trait Selector<Var, Store> {
    fn select(&mut self, store: &Store) -> Option<Var>;
}

pub trait Partitioner<Var, Store, const BRANCHING_FACTOR: usize> {
    fn partition(&mut self, variable: Var, store: &Store) -> [Branch<Store>; BRANCHING_FACTOR];
}

impl<Var, Store, S, P, const BRANCHING_FACTOR: usize> Brancher<Store>
    for Search<Var, S, P, BRANCHING_FACTOR>
where
    S: Selector<Var, Store>,
    P: Partitioner<Var, Store, BRANCHING_FACTOR>,
{
    type Branches = [Branch<Store>; BRANCHING_FACTOR];

    fn branch(&mut self, store: &Store) -> Option<Self::Branches> {
        self.selector
            .select(store)
            .map(|variable| self.partitioner.partition(variable, store))
    }
}
