use limiga_core::{
    domains::DomainStore,
    integer::{BoundedInt, BoundedIntVar, IntEvent},
    lit::Lit,
    propagation::{DomainEvent, LitEvent, Watchable},
    solver::ExtendSolver,
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
