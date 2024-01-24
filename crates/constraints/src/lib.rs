use std::fmt::Debug;

use limiga_core::{
    domains::DomainStore,
    integer::{BoundedInt, BoundedIntVar, IntEvent},
    lit::Lit,
    propagation::{DomainEvent, LitEvent, Watchable},
    solver::{ExtendSolver, Solver},
    storage::StaticIndexer,
};

mod bool_lin_leq;

pub fn bool_lin_leq<VY, Domains, Event>(
    solver: &mut impl ExtendSolver<Domains, Event>,
    x: Box<[Lit]>,
    y: VY,
) -> bool
where
    Event: DomainEvent<LitEvent, IntEvent>,
    VY: BoundedIntVar<Domains, Event> + Watchable<TypedEvent = IntEvent>,
    VY::Dom: BoundedInt,
    Domains: DomainStore<VY::Dom>,
{
    solver.add_propagator(bool_lin_leq::LinearBoolFactory { x, y })
}

pub fn bool_lin_eq<VY, Domains, Event>(
    solver: &mut impl ExtendSolver<Domains, Event>,
    x: Box<[Lit]>,
    y: VY,
) -> bool
where
    Event: DomainEvent<LitEvent, IntEvent>,
    VY: BoundedIntVar<Domains, Event> + Watchable<TypedEvent = IntEvent>,
    VY::Dom: BoundedInt,
    Domains: DomainStore<VY::Dom>,
{
    let neg_x = x.iter().map(|&x_i| !x_i).collect::<Box<[_]>>();

    bool_lin_leq(solver, x, y.clone()) && bool_lin_leq(solver, neg_x, y)
}

/// Post the constraint `(a /\ b) <-> r` in the clausal solver.
pub fn bool_and<Domains, Event>(
    solver: &mut Solver<Domains, Event>,
    a: Lit,
    b: Lit,
    r: Lit,
) -> bool
where
    Event: Copy + Debug + StaticIndexer,
{
    // (a /\ b) -> r
    // equiv (!a \/ !b \/ r)
    solver.add_clause([!a, !b, r]);

    // r -> (a /\ b)
    // equiv (!r \/ (a /\ b))
    // equiv (!r \/ a) /\ (!r \/ b)
    solver.add_clause([!r, a]);
    solver.add_clause([!r, b]);

    true
}
