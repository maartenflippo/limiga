use std::collections::VecDeque;

use crate::{domains::GlobalDomainId, keyed_idx_vec::KeyedVec, propagators::PropagatorId};

#[derive(Default)]
pub(crate) struct PropagatorQueue {
    buffer: VecDeque<PropagatorId>,
    watch_list: KeyedVec<GlobalDomainId, Vec<PropagatorId>>,
}

impl PropagatorQueue {
    pub fn on_new_domain(&mut self, id: GlobalDomainId) {
        self.watch_list.resize(id, vec![]);
    }

    pub fn pop(&mut self) -> Option<PropagatorId> {
        self.buffer.pop_front()
    }

    pub fn react(&mut self, updated_domain: GlobalDomainId) {
        for id in &self.watch_list[updated_domain] {
            self.buffer.push_back(*id);
        }
    }

    pub fn register_watch(&mut self, domain: GlobalDomainId, propagator: PropagatorId) {
        self.watch_list[domain].push(propagator);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_queue_yields_no_next_propagator() {
        let mut queue = PropagatorQueue::default();
        assert_eq!(None, queue.pop());
    }

    #[test]
    fn reacting_to_an_updated_domain_enqueues_propagators() {
        let mut queue = PropagatorQueue::default();

        let domain_id = GlobalDomainId::from_index(0);
        let propagator_id = PropagatorId::from_index(0);

        queue.on_new_domain(domain_id);
        queue.register_watch(domain_id, propagator_id);
        queue.react(domain_id);

        assert_eq!(Some(propagator_id), queue.pop());
    }
}
