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
