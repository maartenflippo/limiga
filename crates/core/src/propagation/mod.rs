mod reason;
mod watch_list;

use std::{marker::PhantomData, ops::Deref};

use crate::{
    assignment::Assignment,
    domains::{DomainId, DomainStore},
    lit::Lit,
    storage::Indexer,
    Conflict,
};
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
    assignment: &'a Assignment,
    domains: &'a mut Domains,
    event: PhantomData<Event>,
}

impl<Domains, Event> Context<'_, Domains, Event> {
    pub fn new<'a>(
        assignment: &'a Assignment,
        domains: &'a mut Domains,
    ) -> Context<'a, Domains, Event> {
        Context {
            assignment,
            domains,
            event: PhantomData,
        }
    }

    pub fn value(&self, lit: PropagatorVar<Lit>) -> Option<bool> {
        self.assignment.value(lit.variable)
    }

    pub fn assign(&mut self, _lit: PropagatorVar<Lit>, _value: bool) -> Result<(), Conflict> {
        todo!()
    }

    pub fn read<Dom>(&self, domain_id: DomainId<Dom>) -> &Dom
    where
        Domains: DomainStore<Dom>,
    {
        &self.domains[domain_id]
    }

    pub fn read_mut<Dom>(&mut self, domain_id: DomainId<Dom>) -> &mut Dom
    where
        Domains: DomainStore<Dom>,
    {
        &mut self.domains[domain_id]
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
    fn index<'slice, Value>(&self, slice: &'slice [Value]) -> &'slice Value {
        &slice[self.0]
    }

    fn index_mut<'slice, Value>(&self, slice: &'slice mut [Value]) -> &'slice mut Value {
        &mut slice[self.0]
    }

    fn get_minimum_len(&self) -> usize {
        self.0 + 1
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

pub trait SDomainEvent<Event>: Indexer + From<Event> + Copy {
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
