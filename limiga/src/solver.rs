use std::collections::VecDeque;

use crate::{
    domains::{DomainId, DomainStore, Domains, GlobalDomainId},
    propagator_queue::PropagatorQueue,
    propagators::{PropagationResult, Propagator, PropagatorStore, RegistrationContext},
    search::{Branch, Brancher},
    IntVar, Variable,
};

#[derive(Default)]
pub struct Solver {
    domains: Domains,
    propagators: PropagatorStore<Domains, PropagatorRegistration>,

    state: State,
    action_queue: VecDeque<Action<Domains>>,
    propagator_queue: PropagatorQueue,
}

pub enum SolveOutcome<'solver, Brancher> {
    Satisfiable(SolutionIterator<'solver, Brancher>),
    Unsatisfiable,
}

pub struct SolutionIterator<'solver, Brancher> {
    solver: &'solver mut Solver,
    brancher: Brancher,
    search_on_next: bool,
}

pub struct Solution<'solver> {
    solver: &'solver Solver,
}

pub struct PropagatorRegistration(Vec<GlobalDomainId>);

enum Action<Store> {
    Branch(Branch<Store>),
    Backtrack,
}

#[derive(Default, PartialEq)]
enum State {
    #[default]
    Finished,
    Solution,
}

impl Solver {
    pub fn new_int_var<Dom>(&mut self, domain: Dom) -> IntVar<Dom, Domains, PropagatorRegistration>
    where
        Domains: DomainStore<Dom>,
    {
        let id = self.domains.alloc(domain);
        self.propagator_queue.on_new_domain(id.global_id());

        id.into()
    }

    pub fn post(
        &mut self,
        propagator: Box<dyn Propagator<Domains, PropagatorRegistration>>,
    ) -> PropagationResult {
        let propagator_id = self.propagators.alloc(propagator);

        let mut ctx = PropagatorRegistration(vec![]);
        let propagator = self.propagators.get_mut(propagator_id);
        propagator.initialize(&mut ctx);
        ctx.finish(&mut self.propagator_queue, propagator_id);

        propagator.propagate(&mut self.domains)?;

        self.propagate()
    }

    pub fn solve<B: Brancher<Domains>>(&mut self, mut brancher: B) -> SolveOutcome<'_, B> {
        self.add_branches(&mut brancher);
        self.next_solution(&mut brancher);

        if self.state == State::Solution {
            SolveOutcome::Satisfiable(SolutionIterator {
                solver: self,
                brancher,
                search_on_next: false,
            })
        } else {
            SolveOutcome::Unsatisfiable
        }
    }

    fn next_solution(&mut self, brancher: &mut impl Brancher<Domains>) {
        while let Some(action) = self.action_queue.pop_front() {
            match action {
                Action::Branch(branch) => {
                    self.domains.push();
                    branch(&mut self.domains);
                    self.react_to_updated_domains();

                    if self.propagate() == PropagationResult::Inconsistent {
                        self.domains.pop();
                        continue;
                    }

                    if !self.add_branches(brancher) {
                        self.state = State::Solution;
                        return;
                    }
                }

                Action::Backtrack => {
                    self.domains.pop();
                }
            }
        }

        self.state = State::Finished;
    }

    fn add_branches(&mut self, brancher: &mut impl Brancher<Domains>) -> bool {
        if let Some(branches) = brancher.branch(&self.domains) {
            self.action_queue.push_front(Action::Backtrack);

            for branch in branches {
                self.action_queue.push_front(Action::Branch(branch));
            }

            true
        } else {
            false
        }
    }

    fn propagate(&mut self) -> PropagationResult {
        while let Some(propagator_id) = self.propagator_queue.pop() {
            let propagator = self.propagators.get_mut(propagator_id);

            if propagator.propagate(&mut self.domains) == PropagationResult::Inconsistent {
                return PropagationResult::Inconsistent;
            }

            self.react_to_updated_domains();
        }

        PropagationResult::Consistent
    }

    fn react_to_updated_domains(&mut self) {
        for updated_domain in self.domains.drain_updated_domains() {
            self.propagator_queue.react(updated_domain);
        }
    }
}

impl<'solver, B: Brancher<Domains>> SolutionIterator<'solver, B> {
    pub fn next_solution(&mut self) -> Option<Solution<'_>> {
        if !self.search_on_next {
            self.search_on_next = true;

            return Some(Solution {
                solver: self.solver,
            });
        }

        if self.solver.state == State::Finished {
            return None;
        }

        self.solver.domains.pop();
        self.solver.next_solution(&mut self.brancher);

        if self.solver.state == State::Solution {
            Some(Solution {
                solver: self.solver,
            })
        } else {
            None
        }
    }
}

impl<'solver> Solution<'solver> {
    pub fn value<Var>(&self, variable: Var) -> Var::Value
    where
        Var: Variable<Domains>,
        Domains: DomainStore<Var::Dom>,
    {
        variable
            .fixed_value(&self.solver.domains)
            .expect("in a solution all variables are fixed")
    }
}

impl<Dom> RegistrationContext<Dom> for PropagatorRegistration {
    fn register(&mut self, domain: DomainId<Dom>) {
        self.0.push(domain.global_id());
    }
}

impl PropagatorRegistration {
    fn finish(
        self,
        propagator_queue: &mut PropagatorQueue,
        propagator_id: crate::propagators::PropagatorId,
    ) {
        for domain_id in self.0 {
            propagator_queue.register_watch(domain_id, propagator_id);
        }
    }
}
