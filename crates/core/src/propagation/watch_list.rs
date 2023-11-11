use std::ops::{Index, IndexMut};

use crate::{
    clause::ClauseRef,
    domains::{Domain, DomainId, UntypedDomainId},
    lit::Lit,
    storage::{Indexer, KeyedVec, StaticIndexer},
};

use super::{DomainEvent, LitEvent, LocalId, PropagatorId, PropagatorVar, SDomainEvent};

pub struct WatchList<Event> {
    literal_watches: KeyedVec<Lit, Vec<LitWatch<Event>>>,
    domain_event_watches: KeyedVec<Event, Vec<PropagatorWatch>>,
}

pub struct PropagatorWatch {
    propagator_id: PropagatorId,
    local_id: LocalId,
}

impl<Event: StaticIndexer> Default for WatchList<Event> {
    fn default() -> Self {
        WatchList {
            literal_watches: KeyedVec::default(),
            domain_event_watches: KeyedVec::with_static_len(),
        }
    }
}

impl<Event> WatchList<Event> {
    pub fn grow_to(&mut self, lit: Lit) {
        self.literal_watches.grow_to(lit);
    }

    pub fn add_lit_watch(&mut self, lit: Lit, watch: LitWatch<Event>) {
        self.literal_watches[lit].push(watch);
    }
}

impl<Event: Indexer> WatchList<Event> {
    pub fn add_event_watch(&mut self, event: Event, watch: PropagatorWatch) {
        self.domain_event_watches[event].push(watch);
    }
}

impl<Event> Index<Lit> for WatchList<Event> {
    type Output = Vec<LitWatch<Event>>;

    fn index(&self, index: Lit) -> &Self::Output {
        &self.literal_watches[index]
    }
}

impl<Event> IndexMut<Lit> for WatchList<Event> {
    fn index_mut(&mut self, index: Lit) -> &mut Self::Output {
        &mut self.literal_watches[index]
    }
}

#[derive(Clone, Copy, Debug)]
pub enum LitWatch<Event> {
    Clause(ClauseRef),

    Propagator {
        propagator_id: PropagatorId,
        local_id: LocalId,
    },

    DomainEvent {
        domain_id: UntypedDomainId,
        event: Event,
    },
}

impl<Event> From<ClauseRef> for LitWatch<Event> {
    fn from(value: ClauseRef) -> Self {
        LitWatch::Clause(value)
    }
}

pub struct VariableRegistrar<'a, Event> {
    propagator: PropagatorId,
    watch_list: &'a mut WatchList<Event>,
}

impl<'a, Event> VariableRegistrar<'a, Event> {
    pub fn new(propagator: PropagatorId, watch_list: &'a mut WatchList<Event>) -> Self {
        VariableRegistrar {
            propagator,
            watch_list,
        }
    }
}

impl<Event> VariableRegistrar<'_, Event> {
    pub fn register<Var>(
        &mut self,
        variable: Var,
        local_id: LocalId,
        event: Var::TypedEvent,
    ) -> PropagatorVar<Var>
    where
        Var: Watchable,
        Event: SDomainEvent<Var::TypedEvent>,
    {
        variable.watch(self.watch_list, self.propagator, local_id, event);
        PropagatorVar { variable, local_id }
    }
}

pub trait Watchable {
    type TypedEvent;

    fn watch<Event>(
        &self,
        watch_list: &mut WatchList<Event>,
        propagator_id: PropagatorId,
        local_id: LocalId,
        event: Self::TypedEvent,
    ) where
        Event: DomainEvent<Self::TypedEvent>;
}

impl Watchable for Lit {
    type TypedEvent = LitEvent;

    fn watch<Event>(
        &self,
        watch_list: &mut WatchList<Event>,
        propagator_id: PropagatorId,
        local_id: LocalId,
        event: Self::TypedEvent,
    ) {
        match event {
            LitEvent::FixedTrue => watch_list.add_lit_watch(
                *self,
                LitWatch::Propagator {
                    propagator_id,
                    local_id,
                },
            ),
            LitEvent::FixedFalse => watch_list.add_lit_watch(
                !*self,
                LitWatch::Propagator {
                    propagator_id,
                    local_id,
                },
            ),
        }
    }
}

impl<Dom> Watchable for DomainId<Dom>
where
    Dom: Domain,
{
    type TypedEvent = Dom::ProducedEvent;

    fn watch<Event>(
        &self,
        watch_list: &mut WatchList<Event>,
        propagator_id: PropagatorId,
        local_id: LocalId,
        event: Self::TypedEvent,
    ) where
        Event: DomainEvent<Self::TypedEvent>,
    {
        watch_list.add_event_watch(
            event.into(),
            PropagatorWatch {
                propagator_id,
                local_id,
            },
        );
    }
}
