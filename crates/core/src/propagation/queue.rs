use bitvec::vec::BitVec;

use crate::storage::Indexer;

use super::PropagatorId;

/// The queue of domain propagators that needs to be executed at the current node. Propagator id's
/// can be pushed onto the queue, but duplicate id's will be ignored.
#[derive(Default)]
pub struct PropagatorQueue {
    present: BitVec,
    queue: Vec<PropagatorId>,
}

impl PropagatorQueue {
    pub fn grow_to(&mut self, propagator_id: PropagatorId) {
        self.present.resize(propagator_id.index() + 1, false);
    }

    /// Push a new propagator into the queue.
    pub fn push(&mut self, propagator_id: PropagatorId) {
        if !self.present[propagator_id.index()] {
            self.present.set(propagator_id.index(), true);
            self.queue.push(propagator_id);
        }
    }

    /// Remove the first propagator id from the queue.
    pub fn pop(&mut self) -> Option<PropagatorId> {
        let id = self.queue.pop()?;
        self.present.set(id.index(), false);
        Some(id)
    }
}
