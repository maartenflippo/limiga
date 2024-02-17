use crate::{
    domains::Conflict,
    lit::Lit,
    propagation::{Context, Explanation},
    variable::Variable,
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

    fn max_lit(&self, ctx: &mut Context<Domains, Event>) -> Lit {
        self.inner.max_lit(ctx)
    }

    fn min_lit(&self, ctx: &mut Context<Domains, Event>) -> Lit {
        self.inner.min_lit(ctx)
    }

    fn set_min(
        &self,
        ctx: &mut Context<Domains, Event>,
        bound: Int,
        explanation: impl Into<Explanation>,
    ) -> Result<(), Conflict> {
        let bound = Int::div_ceil(bound - self.offset, self.scale);

        self.inner.set_min(ctx, bound, explanation)
    }

    fn set_max(
        &self,
        ctx: &mut Context<Domains, Event>,
        bound: Int,
        explanation: impl Into<Explanation>,
    ) -> Result<(), Conflict> {
        let bound = Int::div_floor(bound - self.offset, self.scale);

        self.inner.set_max(ctx, bound, explanation)
    }
}
