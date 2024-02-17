use std::{
    collections::HashMap, fs::File, io::BufReader, path::Path, process::ExitCode, time::Duration,
};

use flatzinc_serde::FlatZinc;
use limiga_constraints::{bool_lin_leq, linear_leq};
use limiga_core::{
    brancher::VsidsBrancher,
    domains::{DomainId, DomainStore, TypedDomainStore},
    integer::{interval_domain::IntInterval, Int, IntEvent},
    lit::Lit,
    propagation::{DomainEvent, LitEvent, SDomainEvent},
    solver::{SolveResult, Solver},
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
) -> anyhow::Result<VariableMap>
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

                        let lower_bound = i32::try_from(lower_bound).map_err(|_| anyhow::anyhow!("the lower bound for {name} does not fit in a 32-bit signed integer"))?;
                        let upper_bound = i32::try_from(upper_bound).map_err(|_| anyhow::anyhow!("the upper bound for {name} does not fit in a 32-bit signed integer"))?;

                        let domain =
                            solver.new_domain(IntInterval::factory(lower_bound, upper_bound));

                        SolverVariable::Int(domain)
                    }

                    flatzinc_serde::Domain::Float(_) => {
                        anyhow::bail!("float domains are not supported");
                    }
                },

                None => anyhow::bail!("unbounded integers are not supported"),
            },

            flatzinc_serde::Type::Float | flatzinc_serde::Type::IntSet => {
                anyhow::bail!("float and set domains are not supported");
            }
        };

        result.insert(name.into(), solver_variable);
    }

    Ok(VariableMap { map: result })
}

fn post_constraints<Domains, Event>(
    fzn: &FlatZinc,
    variables: &VariableMap,
    solver: &mut Solver<Domains, Event>,
) -> anyhow::Result<()>
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
                            anyhow::anyhow!("no array for identifier '{}'", constraint.id)
                        })?
                        .contents
                        .iter()
                        .map(|literal| match literal {
                            flatzinc_serde::Literal::Identifier(element_id) => {
                                variables.resolve_bool_variable(element_id).ok_or_else(|| {
                                    anyhow::anyhow!(
                                        "could not resolve bool variable for {element_id}"
                                    )
                                })
                            }

                            other => anyhow::bail!("expected an identifier, got {other:?}"),
                        })
                        .collect::<Result<_, _>>()?,

                    other => anyhow::bail!("expected an identifier, got {other:?}"),
                };

                let y = fzn.resolve_int_variable_argument(&constraint.args[2], variables)?;

                bool_lin_leq(solver, x, y);
            }

            "int_lin_le" => {
                let terms =
                    fzn.resolve_int_variable_array_argument(&constraint.args[1], variables)?;
                let rhs = fzn.resolve_int_constant_argument(&constraint.args[2])?;

                linear_leq(solver, terms, rhs);
            }

            unsupported => {
                anyhow::bail!("the constraint '{unsupported}' is not supported")
            }
        }
    }

    Ok(())
}

struct VariableMap {
    map: HashMap<String, SolverVariable>,
}

impl VariableMap {
    fn iter(&self) -> std::collections::hash_map::Iter<'_, String, SolverVariable> {
        self.map.iter()
    }

    fn resolve_bool_variable(&self, identifier: &str) -> Option<Lit> {
        self.map
            .get(identifier)
            .and_then(|variable| match variable {
                SolverVariable::Bool(lit) => Some(*lit),
                SolverVariable::Int(_) => None,
            })
    }

    fn resolve_int_variable(&self, identifier: &str) -> Option<DomainId<IntInterval>> {
        self.map
            .get(identifier)
            .and_then(|variable| match variable {
                SolverVariable::Bool(_) => None,
                SolverVariable::Int(domain_id) => Some(domain_id.clone()),
            })
    }
}

trait AstExt {
    fn get_ast(&self) -> &flatzinc_serde::FlatZinc;

    fn resolve_int_constant_argument(
        &self,
        argument: &flatzinc_serde::Argument,
    ) -> anyhow::Result<Int> {
        match argument {
            flatzinc_serde::Argument::Literal(literal) => match literal {
                flatzinc_serde::Literal::Int(int) => Int::try_from(*int).map_err(|_| {
                    anyhow::anyhow!("the value {int} does not fit into our integer representation")
                }),
                other => anyhow::bail!("expected int constant, got {other:?}"),
            },

            other => anyhow::bail!("expected int constant, got {other:?}"),
        }
    }

    fn resolve_int_variable_argument(
        &self,
        argument: &flatzinc_serde::Argument,
        variables: &VariableMap,
    ) -> anyhow::Result<DomainId<IntInterval>> {
        match argument {
            flatzinc_serde::Argument::Literal(literal) => match literal {
                flatzinc_serde::Literal::Identifier(identifier) => {
                    variables.resolve_int_variable(identifier).ok_or_else(|| {
                        anyhow::anyhow!("failed to resolve the integer variable for {identifier}")
                    })
                }

                other => anyhow::bail!("expected identifier, got {other:?}"),
            },

            flatzinc_serde::Argument::Array(_) => {
                anyhow::bail!("expected an identifier, got an array")
            }
        }
    }

    fn resolve_int_variable_array_argument(
        &self,
        argument: &flatzinc_serde::Argument,
        variables: &VariableMap,
    ) -> anyhow::Result<Box<[DomainId<IntInterval>]>> {
        match argument {
            flatzinc_serde::Argument::Literal(flatzinc_serde::Literal::Identifier(identifier)) => {
                self.get_ast()
                    .arrays
                    .get(identifier)
                    .ok_or_else(|| anyhow::anyhow!("no array for identifier '{identifier}'"))?
                    .contents
                    .iter()
                    .map(|literal| match literal {
                        flatzinc_serde::Literal::Identifier(element_id) => {
                            variables.resolve_int_variable(element_id).ok_or_else(|| {
                                anyhow::anyhow!(
                                    "could not resolve integer variable for {element_id}"
                                )
                            })
                        }

                        other => anyhow::bail!("expected an identifier, got {other:?}"),
                    })
                    .collect::<anyhow::Result<_>>()
            }

            other => anyhow::bail!("expected an identifier, got {other:?}"),
        }
    }

    fn resolve_bool_variable_argument(
        &self,
        argument: &flatzinc_serde::Argument,
        variables: &VariableMap,
    ) -> anyhow::Result<Lit> {
        match argument {
            flatzinc_serde::Argument::Literal(literal) => match literal {
                flatzinc_serde::Literal::Identifier(identifier) => {
                    variables.resolve_bool_variable(identifier).ok_or_else(|| {
                        anyhow::anyhow!("failed to resolve the boolean variable for {identifier}")
                    })
                }

                other => anyhow::bail!("expected identifier, got {other:?}"),
            },

            flatzinc_serde::Argument::Array(_) => {
                anyhow::bail!("expected an identifier, got an array")
            }
        }
    }
}

impl AstExt for flatzinc_serde::FlatZinc {
    fn get_ast(&self) -> &flatzinc_serde::FlatZinc {
        self
    }
}
