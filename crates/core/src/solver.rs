use log::trace;

use crate::{
    analysis::ConflictAnalyzer,
    assignment::Assignment,
    brancher::Brancher,
    clause::{ClauseDb, ClauseRef},
    domains::{DomainId, DomainStore},
    implication_graph::ImplicationGraph,
    lit::{Lit, Var},
    preprocessor::{ClausePreProcessor, PreProcessedClause},
    propagation::{
        Context, LitWatch, Propagator, PropagatorFactory, PropagatorId, PropagatorQueue,
        VariableRegistrar, WatchList,
    },
    search_tree::SearchTree,
    storage::{Arena, StaticIndexer},
    termination::Terminator,
    trail::Trail,
};

pub struct Solver<SearchProc, Domains, Event> {
    brancher: SearchProc,

    domains: Domains,
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

impl<SearchProc, Domains, Event> Solver<SearchProc, Domains, Event> {
    pub fn new_domain<Domain>(&mut self, domain: Domain) -> DomainId<Domain>
    where
        Domains: DomainStore<Domain>,
    {
        self.domains.alloc(domain)
    }
}

impl<SearchProc, Domains, Event> Solver<SearchProc, Domains, Event>
where
    SearchProc: Brancher,
    Event: Copy + std::fmt::Debug,
{
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

                let clause = &self.clauses[clause_ref];
                self.watch_list[clause.head[0]].push(clause_ref.into());
                self.watch_list[clause.head[1]].push(clause_ref.into());
                return;
            }

            lits[0]
        };

        if !self.enqueue(root_assignment, ClauseRef::default()) {
            self.state = State::ConflictAtRoot;
        }

        trace!("adding clause [{root_assignment:?}] as assignment");
    }

    pub fn new_lits(&mut self) -> impl Iterator<Item = Lit> + '_ {
        NewLitIterator {
            solver: self,
            has_introduced_new_literal: false,
        }
    }

    fn enqueue(&mut self, lit: Lit, reason: ClauseRef) -> bool {
        if let Some(false) = self.assignment.value(lit) {
            return false;
        }

        self.trail.enqueue(lit);
        self.assignment.assign(lit);
        self.implication_graph.add(lit.var(), reason);
        self.search_tree.register_assignment(lit);

        if reason != ClauseRef::default() {
            assert_eq!(
                lit, self.clauses[reason][0],
                "Propagated literals should be the first literal in the clause."
            );
        }

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

    fn propagate(&mut self) -> Option<ClauseRef> {
        trace!("propagating...");
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
                    LitWatch::DomainEvent { domain_id, event } => todo!(),
                };

                if let Some(conflict) = conflict {
                    // Copy the remaining watches back to the literal.
                    for &constraint in watches.iter().skip(i + 1) {
                        trace!("adding {constraint:?} to the watch list of {false_lit:?}");
                        self.watch_list[false_lit].push(constraint);
                    }

                    self.next_propagation_idx = self.trail.len();

                    return Some(conflict);
                }
            }
        }

        None
    }

    fn propagate_clause(&mut self, clause_ref: ClauseRef, false_lit: Lit) -> bool {
        let lit_to_propagate = {
            let clause = &mut self.clauses[clause_ref];
            trace!("propagating clause {clause:?}");

            // Make sure the false literal is at position 1 in the clause.
            if clause.head[0] == false_lit {
                clause.head.swap(0, 1);
            }

            // If the 0th watch is true, then clause is already satisfied.
            if self.assignment.value(clause.head[0]) == Some(true) {
                trace!("clause is satisfied because of 0th literal");
                self.watch_list[false_lit].push(clause_ref.into());
                return true;
            }

            // Look for a new literal to watch.
            for tail_idx in 0..clause.tail.len() {
                let candidate = clause.tail[tail_idx];
                if self.assignment.value(candidate) != Some(false) {
                    trace!("found new watch literal {candidate:?}");
                    clause.head[1] = candidate;
                    clause.tail[tail_idx] = false_lit;

                    self.watch_list[clause.head[1]].push(clause_ref.into());
                    return true;
                }
            }

            // The clause is unit under the current assignment.
            self.watch_list[false_lit].push(clause_ref.into());
            clause.head[0]
        };

        trace!(
            "propagating {:?} because of {clause_ref:?}",
            lit_to_propagate
        );

        self.enqueue(lit_to_propagate, clause_ref)
    }
}

impl<SearchProc, Domains, Event> Solver<SearchProc, Domains, Event>
where
    SearchProc: Brancher,
    Event: Copy + std::fmt::Debug,
{
    pub fn solve(&mut self, terminator: impl Terminator) -> SolveResult<'_> {
        if self.state == State::ConflictAtRoot {
            return SolveResult::Unsatisfiable;
        }

        while !terminator.should_stop() {
            match self.propagate() {
                Some(conflict) => {
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

                        let clause_ref = if analysis.learned_clause.len() > 1 {
                            self.clauses.add_clause(analysis.learned_clause)
                        } else {
                            ClauseRef::default()
                        };

                        (
                            analysis.learned_clause[0],
                            clause_ref,
                            analysis.backjump_level,
                        )
                    };

                    self.backtrack_to(backjump_level);

                    assert!(
                        self.enqueue(literal_to_enqueue, reason),
                        "conflicting asserting literal"
                    );

                    self.brancher.on_conflict();
                }

                None => {
                    self.trail.push();
                    self.search_tree.branch();

                    if let Some(decision) = self.brancher.next_decision(&self.assignment) {
                        trace!("decided {decision:?}");
                        assert!(
                            self.enqueue(decision, ClauseRef::default()),
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
        let mut variable_registrar = VariableRegistrar::new(slot.id(), &mut self.watch_list);

        let propagator = factory.create(&mut variable_registrar);
        slot.alloc(propagator);

        true
    }
}

struct NewLitIterator<'a, SearchProc, Domains, Event> {
    solver: &'a mut Solver<SearchProc, Domains, Event>,
    has_introduced_new_literal: bool,
}

impl<SearchProc, Domains, Event> Iterator for NewLitIterator<'_, SearchProc, Domains, Event>
where
    SearchProc: Brancher,
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

impl<SearchProc, Domains, Event> Drop for NewLitIterator<'_, SearchProc, Domains, Event> {
    fn drop(&mut self) {
        if self.has_introduced_new_literal {
            let last_var = Var::try_from(self.solver.next_var_code - 1)
                .expect("was created successfully previously as well");

            self.solver.assignment.grow_to(last_var);
            self.solver.implication_graph.grow_to(last_var);
            self.solver.search_tree.grow_to(last_var);
            self.solver.watch_list.grow_to(Lit::positive(last_var));
            self.solver.analyzer.grow_to(last_var);
        }
    }
}
