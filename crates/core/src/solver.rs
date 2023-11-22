use log::trace;

use crate::{
    analysis::ConflictAnalyzer,
    assignment::Assignment,
    brancher::Brancher,
    clause::{ClauseDb, ClauseRef},
    domains::{
        Conflict, DomainFactory, DomainId, DomainStore, GlobalDomainIdPool, UntypedDomainId,
    },
    implication_graph::ImplicationGraph,
    lit::{Lit, Var},
    preprocessor::{ClausePreProcessor, PreProcessedClause},
    propagation::{
        Context, LitWatch, Propagator, PropagatorFactory, PropagatorId, PropagatorQueue, Reason,
        VariableRegistrar, WatchList,
    },
    search_tree::SearchTree,
    storage::{Arena, Indexer, StaticIndexer},
    termination::Terminator,
    trail::Trail,
};

pub struct Solver<SearchProc, Domains, Event> {
    brancher: SearchProc,

    domains: Domains,
    domain_id_pool: GlobalDomainIdPool,

    preprocessor: ClausePreProcessor,
    analyzer: ConflictAnalyzer,
    clauses: ClauseDb,
    implication_graph: ImplicationGraph,
    search_tree: SearchTree,
    state: State,
    propagators: Arena<PropagatorId, Box<dyn Propagator<Domains, Event>>>,
    propagator_queue: PropagatorQueue,

    trail: Trail,
    assignment: Assignment,

    next_propagation_idx: usize,
    watch_list: WatchList<Event>,
    next_var_code: u32,
}

pub trait ExtendSolver<Domains, Event> {
    fn add_propagator(&mut self, factory: impl PropagatorFactory<Domains, Event>) -> bool;
}

pub trait ExtendClausalSolver<Event> {
    type NewLits<'a>: Iterator<Item = Lit>
    where
        Self: 'a;

    fn new_lits(&mut self) -> Self::NewLits<'_>;
    fn add_clause(&mut self, lits: impl IntoIterator<Item = Lit>);

    fn add_domain_watch(&mut self, lit: Lit, event: Event);
}

#[derive(Default, PartialEq, Eq)]
enum State {
    #[default]
    Consistent,
    ConflictAtRoot,
}

impl<SearchProc, Domains, Event> Solver<SearchProc, Domains, Event>
where
    Domains: Default,
    Event: StaticIndexer,
{
    pub fn new(brancher: SearchProc) -> Self {
        Solver {
            brancher,
            domains: Default::default(),
            domain_id_pool: Default::default(),
            clauses: Default::default(),
            search_tree: Default::default(),
            state: Default::default(),
            trail: Default::default(),
            assignment: Default::default(),
            next_propagation_idx: 0,
            watch_list: Default::default(),
            next_var_code: 0,
            preprocessor: Default::default(),
            analyzer: Default::default(),
            implication_graph: Default::default(),
            propagators: Default::default(),
            propagator_queue: Default::default(),
        }
    }
}

impl<SearchProc, Domains, Event> Solver<SearchProc, Domains, Event>
where
    Event: Copy + std::fmt::Debug + StaticIndexer + Indexer,
    SearchProc: Brancher,
{
    pub fn new_domain<Factory>(&mut self, factory: Factory) -> DomainId<Factory::Domain>
    where
        Factory: DomainFactory<Event>,
        Domains: DomainStore<Factory::Domain>,
    {
        let global_id = self.domain_id_pool.next_id();
        self.watch_list.grow_to_domain(global_id);

        let domain = factory.create(&mut DomainFactoryContext {
            solver: self,
            untyped_domain_id: global_id,
        });

        self.domains.alloc(global_id, domain)
    }
}

impl<SearchProc, Domains, Event> Solver<SearchProc, Domains, Event>
where
    SearchProc: Brancher,
    Event: Copy + std::fmt::Debug + StaticIndexer + Indexer,
{
    pub fn new_lits(&mut self) -> NewLitIterator<'_, SearchProc, Domains, Event> {
        NewLitIterator {
            solver: self,
            has_introduced_new_literal: false,
        }
    }

    pub fn add_clause(&mut self, lits: impl IntoIterator<Item = Lit>) {
        if self.state == State::ConflictAtRoot {
            return;
        }

        let root_assignment = {
            let lits = match self.preprocessor.preprocess(lits, &self.assignment) {
                PreProcessedClause::Satisfiable => return,
                PreProcessedClause::Lits(lits) => lits,
            };

            if lits.is_empty() {
                self.state = State::ConflictAtRoot;
                return;
            }

            if lits.len() > 1 {
                let clause_ref = self.clauses.add_clause(lits);
                trace!("adding clause {lits:?} with id {clause_ref:?}");

                self.watch_clause(clause_ref);
                return;
            }

            lits[0]
        };

        if !self.enqueue(root_assignment, Reason::Decision) {
            self.state = State::ConflictAtRoot;
        }

        trace!("adding clause [{root_assignment:?}] as assignment");
    }

    fn watch_clause(&mut self, clause_ref: ClauseRef) {
        trace!("setting up watchers for {clause_ref:?}");
        let clause = &self.clauses[clause_ref];
        self.watch_list[clause[0]].push(clause_ref.into());
        self.watch_list[clause[1]].push(clause_ref.into());
    }

    fn enqueue(&mut self, lit: Lit, reason: Reason) -> bool {
        if let Some(false) = self.assignment.value(lit) {
            return false;
        }

        self.trail.enqueue(lit);
        self.assignment.assign(lit);
        self.implication_graph.add(lit.var(), reason);
        self.search_tree.register_assignment(lit);

        true
    }

    fn backtrack_to(&mut self, decision_level: usize) {
        self.trail.backtrack_to(decision_level).for_each(|lit| {
            self.assignment.unassign(lit);
            self.brancher.on_variable_unassigned(lit.var());
        });

        self.search_tree.cut(decision_level);
        self.next_propagation_idx = self.trail.len();
    }

    fn propagate(&mut self) -> Result<(), ClauseRef> {
        trace!("propagating...");
        self.propagate_propositional()?;

        while let Some(propagator_id) = self.propagator_queue.pop() {
            self.propagate_propagator(propagator_id)?;
            self.propagate_propositional()?;
        }

        Ok(())
    }

    fn propagate_propagator(&mut self, propagator_id: PropagatorId) -> Result<(), ClauseRef> {
        trace!("propagating propagator {propagator_id:?}...");
        let propagator = &mut self.propagators[propagator_id];
        let mut ctx = Context::new(
            &mut self.assignment,
            &mut self.trail,
            &mut self.implication_graph,
            &mut self.search_tree,
            &mut self.domains,
        );

        propagator
            .propagate(&mut ctx)
            .map_err(|conflict| self.create_conflict_clause(conflict))
    }

    fn propagate_propositional(&mut self) -> Result<(), ClauseRef> {
        trace!("propagating propositional trail...");
        while self.next_propagation_idx < self.trail.len() {
            let trail_lit = self.trail[self.next_propagation_idx];
            let false_lit = !trail_lit;
            self.next_propagation_idx += 1;

            trace!("processing {trail_lit:?}");

            let watches = std::mem::take(&mut self.watch_list[false_lit]);

            trace!("watched constraints {watches:?}");

            for i in 0..watches.len() {
                let watch = watches[i];

                let conflict = match watch {
                    LitWatch::Clause(clause_ref) => {
                        if !self.propagate_clause(clause_ref, false_lit) {
                            Some(clause_ref)
                        } else {
                            None
                        }
                    }
                    LitWatch::Propagator {
                        propagator_id,
                        local_id: _,
                    } => {
                        self.propagator_queue.push(propagator_id);
                        None
                    }
                    LitWatch::DomainEvent { domain_id, event } => {
                        self.watch_list[(domain_id, event)]
                            .iter()
                            .for_each(|watch| self.propagator_queue.push(watch.propagator_id));
                        None
                    }
                };

                if let Some(conflict) = conflict {
                    // Copy the remaining watches back to the literal.
                    for &constraint in watches.iter().skip(i + 1) {
                        trace!("adding {constraint:?} to the watch list of {false_lit:?}");
                        self.watch_list[false_lit].push(constraint);
                    }

                    self.next_propagation_idx = self.trail.len();

                    return Err(conflict);
                }
            }
        }

        Ok(())
    }

    fn propagate_clause(&mut self, clause_ref: ClauseRef, false_lit: Lit) -> bool {
        let lit_to_propagate = {
            let is_learned = self.clauses.is_learned(clause_ref);
            let clause = &mut self.clauses[clause_ref];
            trace!("propagating clause {clause:?} (is_learned: {is_learned})");

            // Make sure the false literal is at position 1 in the clause.
            if clause[0] == false_lit {
                clause.swap(0, 1);
            }

            // If the 0th watch is true, then clause is already satisfied.
            if self.assignment.value(clause[0]) == Some(true) {
                trace!("clause is satisfied because of 0th literal");
                self.watch_list[false_lit].push(clause_ref.into());
                return true;
            }

            // Look for a new literal to watch.
            for idx in 2..clause.len() {
                let candidate = clause[idx];
                if self.assignment.value(candidate) != Some(false) {
                    trace!("found new watch literal {candidate:?}");
                    clause.swap(1, idx);

                    self.watch_list[clause[1]].push(clause_ref.into());
                    return true;
                }
            }

            // The clause is unit under the current assignment.
            self.watch_list[false_lit].push(clause_ref.into());
            clause[0]
        };

        trace!(
            "propagating {:?} because of {clause_ref:?}",
            lit_to_propagate
        );

        self.enqueue(lit_to_propagate, clause_ref.into())
    }

    fn create_conflict_clause(&mut self, conflict: Conflict) -> ClauseRef {
        assert_eq!(Some(false), self.assignment.value(conflict.lit));

        let other_reason = self.implication_graph.reason(conflict.lit.var());

        let mut lits = conflict.explanation.into_vec();

        match other_reason {
            Reason::Decision => {}
            Reason::Clause(clause_ref) => lits.extend(self.clauses[*clause_ref].iter()),
            Reason::Explanation(other_lits) => lits.extend(other_lits.iter()),
        }

        lits.iter_mut().for_each(|lit| *lit = !*lit);

        self.clauses.add_explanation_clause(lits)
    }
}

impl<SearchProc, Domains, Event> Solver<SearchProc, Domains, Event>
where
    SearchProc: Brancher,
    Event: Copy + std::fmt::Debug + StaticIndexer + Indexer,
{
    pub fn solve(&mut self, terminator: impl Terminator) -> SolveResult<'_> {
        if self.state == State::ConflictAtRoot {
            return SolveResult::Unsatisfiable;
        }

        while !terminator.should_stop() {
            match self.propagate() {
                Err(conflict) => {
                    trace!("conflict at dl {}", self.search_tree.depth());

                    if self.search_tree.is_at_root() {
                        return SolveResult::Unsatisfiable;
                    }

                    let (literal_to_enqueue, reason, backjump_level) = {
                        let analysis = self.analyzer.analyze(
                            conflict,
                            &self.clauses,
                            &self.implication_graph,
                            &self.search_tree,
                            &self.trail,
                            &mut self.brancher,
                        );

                        trace!("learned clause {:?}", analysis.learned_clause);

                        let clause_ref = if analysis.learned_clause.len() > 1 {
                            self.clauses
                                .add_learned_clause(analysis.learned_clause)
                                .into()
                        } else {
                            Reason::Decision
                        };

                        (
                            analysis.learned_clause[0],
                            clause_ref,
                            analysis.backjump_level,
                        )
                    };

                    if let Reason::Clause(clause_ref) = reason {
                        self.watch_clause(clause_ref);
                    }

                    self.backtrack_to(backjump_level);

                    assert!(
                        self.enqueue(literal_to_enqueue, reason),
                        "conflicting asserting literal"
                    );

                    self.brancher.on_conflict();
                }

                Ok(()) => {
                    self.trail.push();
                    self.search_tree.branch();

                    if let Some(decision) = self.brancher.next_decision(&self.assignment) {
                        trace!("decided {decision:?}");
                        assert!(
                            self.enqueue(decision, Reason::Decision),
                            "decided already assigned literal"
                        );
                    } else {
                        return SolveResult::Satisfiable(Solution {
                            assignment: &mut self.assignment,
                            next_new_var_code: self.next_var_code,
                        });
                    }
                }
            }
        }

        SolveResult::Unknown
    }
}

pub enum SolveResult<'solver> {
    /// A solution has been found for the formula.
    Satisfiable(Solution<'solver>),
    /// No solution exists for the formula.
    Unsatisfiable,
    /// The solver was interrupted before reaching a conclusion.
    Unknown,
}

pub struct Solution<'assignment> {
    assignment: &'assignment mut Assignment,
    next_new_var_code: u32,
}

impl Solution<'_> {
    pub fn value(&self, var: Var) -> bool {
        self.assignment.value(Lit::positive(var)).unwrap()
    }

    pub fn vars(&self) -> impl Iterator<Item = Var> + '_ {
        (0..self.next_new_var_code).map(|code| Var::try_from(code).unwrap())
    }
}

impl<SearchProc, Domains, Event> ExtendSolver<Domains, Event>
    for Solver<SearchProc, Domains, Event>
{
    fn add_propagator(&mut self, factory: impl PropagatorFactory<Domains, Event>) -> bool {
        let slot = self.propagators.new_ref();
        self.propagator_queue.grow_to(slot.id());
        let mut variable_registrar = VariableRegistrar::new(slot.id(), &mut self.watch_list);

        let propagator = factory.create(&mut variable_registrar);
        slot.alloc(propagator);

        true
    }
}

struct DomainFactoryContext<'a, SearchProc, Domains, Event> {
    solver: &'a mut Solver<SearchProc, Domains, Event>,
    untyped_domain_id: UntypedDomainId,
}

impl<SearchProc, Domains, Event> ExtendClausalSolver<Event>
    for DomainFactoryContext<'_, SearchProc, Domains, Event>
where
    Event: Copy + std::fmt::Debug + StaticIndexer + Indexer,
    SearchProc: Brancher,
{
    type NewLits<'a> = NewLitIterator<'a, SearchProc, Domains, Event>
    where
        Self: 'a;

    fn new_lits(&mut self) -> Self::NewLits<'_> {
        self.solver.new_lits()
    }

    fn add_clause(&mut self, lits: impl IntoIterator<Item = Lit>) {
        self.solver.add_clause(lits)
    }

    fn add_domain_watch(&mut self, lit: Lit, event: Event) {
        self.solver.watch_list.add_lit_watch(
            lit,
            LitWatch::DomainEvent {
                domain_id: self.untyped_domain_id,
                event,
            },
        )
    }
}

pub struct NewLitIterator<'a, SearchProc, Domains, Event>
where
    Event: StaticIndexer,
{
    solver: &'a mut Solver<SearchProc, Domains, Event>,
    has_introduced_new_literal: bool,
}

impl<SearchProc, Domains, Event> Iterator for NewLitIterator<'_, SearchProc, Domains, Event>
where
    SearchProc: Brancher,
    Event: StaticIndexer,
{
    type Item = Lit;

    fn next(&mut self) -> Option<Self::Item> {
        let var = Var::try_from(self.solver.next_var_code).expect("valid var code");
        let lit = Lit::positive(var);
        self.solver.brancher.on_new_var(var);

        self.solver.next_var_code += 1;
        self.has_introduced_new_literal = true;

        Some(lit)
    }
}

impl<SearchProc, Domains, Event> Drop for NewLitIterator<'_, SearchProc, Domains, Event>
where
    Event: StaticIndexer,
{
    fn drop(&mut self) {
        if self.has_introduced_new_literal {
            let last_var = Var::try_from(self.solver.next_var_code - 1)
                .expect("was created successfully previously as well");

            self.solver.assignment.grow_to(last_var);
            self.solver.implication_graph.grow_to(last_var);
            self.solver.search_tree.grow_to(last_var);
            self.solver.watch_list.grow_to_lit(Lit::positive(last_var));
            self.solver.analyzer.grow_to(last_var);
        }
    }
}
