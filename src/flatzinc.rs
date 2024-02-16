use std::{
    collections::HashMap, fs::File, io::BufReader, path::Path, process::ExitCode, time::Duration,
};

use flatzinc_serde::FlatZinc;
use limiga_constraints::bool_lin_leq;
use limiga_core::{
    brancher::VsidsBrancher,
    domains::{DomainId, DomainStore, TypedDomainStore},
    integer::{interval_domain::IntInterval, IntEvent},
    lit::Lit,
    propagation::{DomainEvent, LitEvent, SDomainEvent},
    solver::{ExtendSolver, SolveResult, Solver},
    storage::{Indexer, StaticIndexer},
    termination::TimeBudget,
};

use crate::termination::{OrTerminator, SignalTerminator};

pub fn solve(path: impl AsRef<Path>, timeout: Option<Duration>) -> ExitCode {
    let path = path.as_ref();

    let Ok(open) = File::open(path) else {
        eprintln!("Failed to open {}", path.display());
        return ExitCode::FAILURE;
    };

    let reader = BufReader::new(open);
    let fzn = match serde_json::from_reader::<_, FlatZinc>(reader) {
        Ok(fzn) => fzn,
        Err(e) => {
            eprintln!("Failed to parse flatzinc.");
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };

    let mut solver: Solver<TypedDomainStore<IntInterval>, SolverEvent> = Solver::default();
    let variables = match create_variables(&fzn, &mut solver) {
        Ok(variables) => variables,
        Err(e) => {
            eprintln!("Failed to parse flatzinc.");
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };

    if let Err(e) = post_constraints(&fzn, &variables, &mut solver) {
        eprintln!("Failed to parse flatzinc.");
        eprintln!("{e}");
        return ExitCode::FAILURE;
    };

    let timer = timeout
        .map(TimeBudget::starting_now)
        .unwrap_or(TimeBudget::infinite());

    let signal_terminator = SignalTerminator::register();
    let terminator = OrTerminator::new(timer, signal_terminator);
    let brancher = VsidsBrancher::new(0.95);

    match solver.solve(terminator, brancher) {
        SolveResult::Satisfiable(solution) => {
            for (name, variable) in variables.iter() {
                let value = match variable {
                    SolverVariable::Int(domain) => {
                        format!("{}", solution.domain_value(domain.clone()))
                    }

                    SolverVariable::Bool(lit) => {
                        format!("{}", solution.value(lit.var()) == lit.is_positive())
                    }
                };

                println!("{name} = {value};");
            }

            println!("----------");
            ExitCode::SUCCESS
        }
        SolveResult::Unsatisfiable => {
            println!("=====UNSATISFIABLE=====");
            ExitCode::SUCCESS
        }
        SolveResult::Unknown => {
            println!("=====UNKNOWN=====");
            ExitCode::SUCCESS
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SolverEvent {
    LitEvent(LitEvent),
    IntEvent(IntEvent),
}

impl From<LitEvent> for SolverEvent {
    fn from(value: LitEvent) -> Self {
        SolverEvent::LitEvent(value)
    }
}

impl From<IntEvent> for SolverEvent {
    fn from(value: IntEvent) -> Self {
        SolverEvent::IntEvent(value)
    }
}

impl Indexer for SolverEvent {
    fn index(&self) -> usize {
        match *self {
            SolverEvent::LitEvent(LitEvent::FixedTrue) => 0,
            SolverEvent::LitEvent(LitEvent::FixedFalse) => 1,
            SolverEvent::IntEvent(IntEvent::LowerBound) => 2,
            SolverEvent::IntEvent(IntEvent::UpperBound) => 3,
        }
    }
}

impl SDomainEvent<LitEvent> for SolverEvent {
    fn is(self, event: LitEvent) -> bool {
        matches!(self, SolverEvent::LitEvent(e) if e == event)
    }
}

impl SDomainEvent<IntEvent> for SolverEvent {
    fn is(self, event: IntEvent) -> bool {
        matches!(self, SolverEvent::IntEvent(e) if e == event)
    }
}

impl StaticIndexer for SolverEvent {
    fn get_len() -> usize {
        4
    }
}

enum SolverVariable {
    Int(DomainId<IntInterval>),
    Bool(Lit),
}

fn create_variables<Domains>(
    ast: &FlatZinc,
    solver: &mut Solver<Domains, SolverEvent>,
) -> Result<HashMap<String, SolverVariable>, Box<str>>
where
    Domains: DomainStore<IntInterval>,
{
    let mut result = HashMap::new();

    for (name, variable) in ast.variables.iter() {
        let solver_variable = match variable.ty {
            flatzinc_serde::Type::Bool => {
                let lit = solver.new_lits().next().unwrap();
                SolverVariable::Bool(lit)
            }

            flatzinc_serde::Type::Int => match variable.domain {
                Some(ref domain) => match domain {
                    flatzinc_serde::Domain::Int(ranges) => {
                        let lower_bound = *ranges.lower_bound().expect("non-empty domain");
                        let upper_bound = *ranges.upper_bound().expect("non-empty domain");

                        let lower_bound = i32::try_from(lower_bound).map_err(|_| format!("the lower bound for {name} does not fit in a 32-bit signed integer"))?;
                        let upper_bound = i32::try_from(upper_bound).map_err(|_| format!("the upper bound for {name} does not fit in a 32-bit signed integer"))?;

                        let domain =
                            solver.new_domain(IntInterval::factory(lower_bound, upper_bound));

                        SolverVariable::Int(domain)
                    }

                    flatzinc_serde::Domain::Float(_) => {
                        return Err("float domains are not supported".into());
                    }
                },

                None => return Err("unbounded integers are not supported".into()),
            },

            flatzinc_serde::Type::Float | flatzinc_serde::Type::IntSet => {
                return Err("float and set domains are not supported".into());
            }
        };

        result.insert(name.into(), solver_variable);
    }

    Ok(result)
}

fn post_constraints<Domains, Event>(
    fzn: &FlatZinc,
    variables: &HashMap<String, SolverVariable>,
    solver: &mut impl ExtendSolver<Domains, Event>,
) -> Result<(), Box<str>>
where
    Domains: DomainStore<IntInterval>,
    Event: DomainEvent<LitEvent, IntEvent>,
{
    for constraint in fzn.constraints.iter() {
        match constraint.id.as_str() {
            "bool_lin_le" => {
                let x = match &constraint.args[1] {
                    flatzinc_serde::Argument::Literal(flatzinc_serde::Literal::Identifier(
                        identifier,
                    )) => fzn
                        .arrays
                        .get(identifier)
                        .ok_or_else(|| {
                            Box::from(format!("no array for identifier '{}'", constraint.id))
                        })?
                        .contents
                        .iter()
                        .map(|literal| match literal {
                            flatzinc_serde::Literal::Identifier(element_id) => {
                                match variables.get(element_id).unwrap() {
                                    SolverVariable::Int(_) => panic!("no domain id expected"),
                                    SolverVariable::Bool(lit) => *lit,
                                }
                            }

                            other => panic!("expected identifier, got {other:?}"),
                        })
                        .collect(),

                    other => return Err(format!("expected an identifier, got {other:?}").into()),
                };

                let y = match &constraint.args[2] {
                    flatzinc_serde::Argument::Literal(literal) => match literal {
                        flatzinc_serde::Literal::Identifier(identifier) => {
                            match variables.get(identifier).unwrap() {
                                SolverVariable::Bool(_) => panic!("no literal expected"),
                                SolverVariable::Int(domain_id) => domain_id.clone(),
                            }
                        }

                        other => panic!("expected identifier, got {other:?}"),
                    },

                    flatzinc_serde::Argument::Array(_) => {
                        return Err("expected an identifier, got an array".into())
                    }
                };

                bool_lin_leq(solver, x, y);
            }

            unsupported => {
                return Err(format!("the constraint '{unsupported}' is not supported").into());
            }
        }
    }

    Ok(())
}
