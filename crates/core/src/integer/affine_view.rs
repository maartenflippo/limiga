use crate::{
    domains::Conflict,
    propagation::{Context, Explanation},
    variable::Variable, atom::Atom, 
};

use super::{BoundedIntVar, Int};

/// Perform an affine transformation to a variable.
#[derive(Clone)]
pub struct Affine<Var> {
    scale: Int,
    offset: Int,
    inner: Var,
}

impl<Var> Affine<Var> {
    /// Create a new variable which is an affine transformation of the given variable.
    pub fn new(scale: Int, offset: Int, inner: Var) -> Self {
        Affine {
            scale,
            offset,
            inner,
        }
    }

    /// Create a new variable which scales the given variable.
    pub fn with_scale(scale: Int, inner: Var) -> Self {
        Affine {
            scale,
            offset: 0,
            inner,
        }
    }

    /// Create a new variable which offsets the given variable.
    pub fn with_offset(offset: Int, inner: Var) -> Self {
        Affine {
            scale: 1,
            offset,
            inner,
        }
    }
}

impl<Var> Variable for Affine<Var>
where
    Var: Variable,
{
    type Dom = Var::Dom;
}

impl<Var, Domains, Event> BoundedIntVar<Domains, Event> for Affine<Var>
where
    Var: BoundedIntVar<Domains, Event>,
{
    fn max(&self, ctx: &mut Context<Domains, Event>) -> Int {
        self.inner.max(ctx) * self.scale + self.offset
    }

    fn min(&self, ctx: &mut Context<Domains, Event>) -> Int {
        self.inner.min(ctx) * self.scale + self.offset
    }

    fn upper_bound_atom(&self, bound: Int) -> Box<dyn Atom<Domains>> {
        if self.scale >= 0 {
            let bound = Int::div_floor(bound - self.offset, self.scale);
            self.inner.upper_bound_atom(bound)
        } else {
            let bound = Int::div_ceil(bound - self.offset, self.scale);
            self.inner.lower_bound_atom(bound)
        }
    }

    fn lower_bound_atom(&self, bound: Int) -> Box<dyn Atom<Domains>> {
        if self.scale >= 0 {
            let bound = Int::div_ceil(bound - self.offset, self.scale);
            self.inner.lower_bound_atom(bound)
        } else {
            let bound = Int::div_floor(bound - self.offset, self.scale);
            self.inner.upper_bound_atom(bound)
        }
    }

    fn set_min(
        &self,
        ctx: &mut Context<Domains, Event>,
        bound: Int,
        explanation: impl Into<Explanation<Domains>>,
    ) -> Result<(), Conflict<Domains>> {
        let bound = Int::div_ceil(bound - self.offset, self.scale);

        self.inner.set_min(ctx, bound, explanation)
    }

    fn set_max(
        &self,
        ctx: &mut Context<Domains, Event>,
        bound: Int,
        explanation: impl Into<Explanation<Domains>>,
    ) -> Result<(), Conflict<Domains>> {
        let bound = Int::div_floor(bound - self.offset, self.scale);

        self.inner.set_max(ctx, bound, explanation)
    }
}
