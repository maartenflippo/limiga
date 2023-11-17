mod queue;
mod reason;
mod watch_list;

use std::{marker::PhantomData, ops::Deref};

use crate::{
    assignment::Assignment,
    domains::{Conflict, DomainId, DomainStore, EnqueueDomainLit},
    implication_graph::ImplicationGraph,
    lit::Lit,
    search_tree::SearchTree,
    storage::{Indexer, StaticIndexer},
    trail::Trail,
};
pub use queue::*;
pub use reason::*;
pub use watch_list::*;

/// A propagator factory is responsible for constructing a propagator and registering it to watch
/// for the events it requires.
pub trait PropagatorFactory<Domains, Event> {
    /// Creates a new instances of a propagator.
    ///
    /// The given registrar should be used to register what events the propagator is interested in.
    fn create(
        self,
        registrar: &mut VariableRegistrar<'_, Event>,
    ) -> Box<dyn Propagator<Domains, Event>>;
}

#[derive(Clone, Copy)]
pub struct PropagatorVar<V> {
    pub variable: V,
    pub local_id: LocalId,
}

impl<V> Deref for PropagatorVar<V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &self.variable
    }
}

pub trait Propagator<Domains, Event> {
    /// Called when an event occurred for the given variable. The propagator should indicate
    /// whether it should be enqueued based on the event.
    ///
    /// By default the propagator will be enqueued for all events that it subscribed to.
    fn on_event(&mut self, _variable: LocalId, _event: Event) -> bool {
        true
    }

    /// Should remove as many values from the domains it is registered for as it can.
    fn propagate(&mut self, ctx: &mut Context<Domains, Event>) -> Result<(), Conflict>;
}

pub struct Context<'a, Domains, Event> {
    assignment: &'a mut Assignment,
    trail: &'a mut Trail,
    implication_graph: &'a mut ImplicationGraph,
    domains: &'a mut Domains,
    search_tree: &'a mut SearchTree,
    event: PhantomData<Event>,
}

impl<Domains, Event> Context<'_, Domains, Event> {
    pub fn new<'a>(
        assignment: &'a mut Assignment,
        trail: &'a mut Trail,
        implication_graph: &'a mut ImplicationGraph,
        search_tree: &'a mut SearchTree,
        domains: &'a mut Domains,
    ) -> Context<'a, Domains, Event> {
        Context {
            assignment,
            trail,
            implication_graph,
            domains,
            search_tree,
            event: PhantomData,
        }
    }

    pub fn value(&self, lit: PropagatorVar<Lit>) -> Option<bool> {
        self.assignment.value(lit.variable)
    }

    pub fn assign(
        &mut self,
        lit: PropagatorVar<Lit>,
        value: bool,
        explanation: impl Into<Explanation>,
    ) -> Result<(), Conflict> {
        let lit = if value { lit.variable } else { !lit.variable };

        let mut enqueue_lit = PropositionalState {
            assignment: self.assignment,
            trail: self.trail,
            implication_graph: self.implication_graph,
            search_tree: self.search_tree,
        };

        enqueue_lit.enqueue(lit, explanation.into())
    }

    pub fn read<Dom>(&self, domain_id: DomainId<Dom>) -> &Dom
    where
        Domains: DomainStore<Dom>,
    {
        &self.domains[domain_id]
    }

    pub fn read_mut<Dom>(
        &mut self,
        domain_id: DomainId<Dom>,
    ) -> (&mut Dom, impl EnqueueDomainLit + '_)
    where
        Domains: DomainStore<Dom>,
    {
        let enqueue_lit = PropositionalState {
            assignment: self.assignment,
            trail: self.trail,
            implication_graph: self.implication_graph,
            search_tree: self.search_tree,
        };
        (&mut self.domains[domain_id], enqueue_lit)
    }
}

pub struct PropositionalState<'a> {
    assignment: &'a mut Assignment,
    trail: &'a mut Trail,
    implication_graph: &'a mut ImplicationGraph,
    search_tree: &'a mut SearchTree,
}

impl EnqueueDomainLit for PropositionalState<'_> {
    fn enqueue(&mut self, lit: Lit, explanation: Explanation) -> Result<(), Conflict> {
        if self.assignment.value(lit) == Some(false) {
            return Err(Conflict { lit, explanation });
        }

        self.trail.enqueue(lit);
        self.implication_graph
            .add(lit.var(), Reason::from_explanation(lit, explanation));
        self.search_tree.register_assignment(lit);
        self.assignment.assign(lit);

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct PropagatorId(usize);

impl From<usize> for PropagatorId {
    fn from(value: usize) -> Self {
        PropagatorId(value)
    }
}

impl Indexer for PropagatorId {
    fn index(&self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct LocalId(u32);

impl From<u32> for LocalId {
    fn from(value: u32) -> Self {
        LocalId(value)
    }
}

pub trait SDomainEvent<Event>: Indexer + StaticIndexer + From<Event> + Copy {
    fn is(self, evt: Event) -> bool;
}

pub trait DomainEvent<E1, E2 = E1, E3 = E1>:
    SDomainEvent<E1> + SDomainEvent<E2> + SDomainEvent<E3>
{
}

impl<Event, E1, E2, E3> DomainEvent<E1, E2, E3> for Event where
    Event: SDomainEvent<E1> + SDomainEvent<E2> + SDomainEvent<E3>
{
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LitEvent {
    FixedTrue,
    FixedFalse,
}
