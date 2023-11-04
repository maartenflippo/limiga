use std::marker::PhantomData;

use crate::lit::Lit;

/// A propagator factory is responsible for constructing a propagator and registering it to watch
/// for the events it requires.
pub trait PropagatorFactory<Event> {
    /// The propagator that is constructed by this factory.
    type Output: Propagator<Event>;

    /// Creates a new instances of a propagator.
    ///
    /// The given registrar should be used to register what events the propagator is interested in.
    fn create(self, registrar: &mut VariableRegistrar<Event>) -> Self::Output;
}

pub struct VariableRegistrar<SolverEvent> {
    event: PhantomData<SolverEvent>,
}

pub struct PropagatorVar<V> {
    var: V,
    id: LocalId,
}

impl<SolverEvent> Default for VariableRegistrar<SolverEvent> {
    fn default() -> Self {
        todo!()
    }
}

impl<SolverEvent> VariableRegistrar<SolverEvent> {
    pub fn register<Var, Event>(
        &mut self,
        variable: Var,
        event: Event,
        local_id: LocalId,
    ) -> PropagatorVar<Var>
    where
        Var: ProducesEvent<Event>,
        Event: Into<SolverEvent>,
    {
        PropagatorVar {
            var: variable,
            id: local_id,
        }
    }
}

pub trait Propagator<Event> {
    /// Called when an event occurred for the given variable. The propagator should indicate
    /// whether it should be enqueued based on the event.
    ///
    /// By default the propagator will be enqueued for all events that it subscribed to.
    fn on_event(&mut self, _variable: LocalId, _event: Event) -> bool {
        true
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct PropagatorId(usize);

impl From<usize> for PropagatorId {
    fn from(value: usize) -> Self {
        PropagatorId(value)
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct LocalId(u32);

impl From<u32> for LocalId {
    fn from(value: u32) -> Self {
        LocalId(value)
    }
}

pub trait SDomainEvent<Event>: From<Event> + Copy {
    fn is(self, evt: Event) -> bool;
}

pub trait DomainEvent<E1, E2 = E1, E3 = E1>:
    SDomainEvent<E1> + SDomainEvent<E2> + SDomainEvent<E3>
{
}

pub trait ProducesEvent<Event> {}

impl ProducesEvent<LitFixedEvent> for Lit {}

/// The event which indicates a literal has been fixed.
pub struct LitFixedEvent;
/// The upper bound of the domain was tightened.
pub struct UpperBoundEvent;
