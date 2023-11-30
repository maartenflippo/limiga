pub mod ast;

use std::io::{self, BufRead, BufReader, Read};

use pest::{iterators::Pair, Parser};
use pest_derive::Parser;
use thiserror::Error;

#[derive(Parser)]
#[grammar = "flatzinc.pest"]
struct FlatZincParser;

#[derive(Debug, Error)]
pub enum FznError {
    #[error("failed to read source")]
    Io(#[from] io::Error),

    #[error("syntax error: {0}")]
    Syntax(#[from] Box<pest::error::Error<Rule>>),
}

/// Parse a flatzinc source into an AST. The parser operates under the assumption that each model
/// item is on a separate line, which matches how the minizinc toolchain produces flatzinc.
pub fn parse(source: impl Read) -> impl Iterator<Item = Result<ast::ModelItem, FznError>> {
    let reader = BufReader::new(source);

    reader
        .lines()
        .enumerate()
        .map::<Result<ast::ModelItem, FznError>, _>(|(idx, line)| {
            let line_number = idx + 1;
            let line = line?;

            let model_item = FlatZincParser::parse(Rule::model_item, line.as_str())
                .map_err(|mut err| {
                    let line_col = match err.line_col {
                        pest::error::LineColLocation::Pos((_, col)) => {
                            pest::error::LineColLocation::Pos((line_number, col))
                        }
                        pest::error::LineColLocation::Span((_, start_col), (_, end_col)) => {
                            pest::error::LineColLocation::Span(
                                (line_number, start_col),
                                (line_number, end_col),
                            )
                        }
                    };

                    err.line_col = line_col;
                    Box::new(err)
                })?
                .next()
                .expect("exactly one rule");

            compile_model_item(model_item)
        })
}

fn compile_model_item(model_item: Pair<Rule>) -> Result<ast::ModelItem, FznError> {
    assert_eq!(Rule::model_item, model_item.as_rule());

    let model_item = model_item.into_inner().next().expect("exactly one rule");

    match model_item.as_rule() {
        Rule::parameter_declaration => compile_parameter_declaration(model_item),
        Rule::variable_declaration => compile_variable_declaration(model_item),

        _ => unreachable!(),
    }
}

fn compile_variable_declaration(
    variable_declaration: Pair<'_, Rule>,
) -> Result<ast::ModelItem, FznError> {
    assert_eq!(Rule::variable_declaration, variable_declaration.as_rule());

    let mut components = variable_declaration.into_inner();

    let first = components.next().expect("missing type rule");

    match first.as_rule() {
        Rule::variable_type => compile_single_variable(first, components),
        Rule::index_set => compile_variable_array(first, components),
        _ => unreachable!(),
    }
}

fn compile_single_variable(
    type_rule: Pair<'_, Rule>,
    mut components: pest::iterators::Pairs<'_, Rule>,
) -> Result<ast::ModelItem, FznError> {
    let identifier_rule = components.next().expect("missing identifier rule");

    let domain = compile_domain(type_rule);
    let identifier = compile_identifier(identifier_rule);

    match domain {
        Domain::Int(domain) => Ok(ast::ModelItem::Variable(ast::Variable::IntVariable(
            ast::SingleVariable { identifier, domain },
        ))),

        Domain::Bool => Ok(ast::ModelItem::Variable(ast::Variable::BoolVariable(
            ast::SingleVariable {
                identifier,
                domain: (),
            },
        ))),
    }
}

fn compile_variable_array(
    index_set_rule: Pair<'_, Rule>,
    mut components: pest::iterators::Pairs<'_, Rule>,
) -> Result<ast::ModelItem, FznError> {
    assert_eq!(Rule::index_set, index_set_rule.as_rule());

    let _num_elements = compile_index_set(index_set_rule);

    let domain_rule = components
        .next()
        .expect("missing domain for variable array declaration");
    let identifier_rule = components
        .next()
        .expect("missing variable array identifier");
    let array_rule = components.next().expect("missing array definition");

    let domain = compile_domain(domain_rule);
    let identifier = compile_identifier(identifier_rule);

    match domain {
        Domain::Int(_) => {
            let variables = array_rule
                .into_inner()
                .map(|basic_expression| {
                    compile_basic_expression(basic_expression, compile_int_literal)
                })
                .collect::<Box<[_]>>();

            Ok(ast::ModelItem::Variable(ast::Variable::ArrayOfIntVariable(
                ast::VariableArray {
                    identifier,
                    variables,
                },
            )))
        }
        Domain::Bool => {
            let variables = array_rule
                .into_inner()
                .map(|basic_expression| {
                    compile_basic_expression(basic_expression, compile_bool_literal)
                })
                .collect::<Box<[_]>>();

            Ok(ast::ModelItem::Variable(
                ast::Variable::ArrayOfBoolVariable(ast::VariableArray {
                    identifier,
                    variables,
                }),
            ))
        }
    }
}

fn compile_basic_expression<Value>(
    basic_expression: Pair<'_, Rule>,
    value_parser: fn(Pair<'_, Rule>) -> Value,
) -> ast::IdentifierOr<Value> {
    assert_eq!(Rule::basic_expression, basic_expression.as_rule());

    let inner = basic_expression
        .into_inner()
        .next()
        .expect("missing contents for basic expression");

    match inner.as_rule() {
        Rule::identifier => ast::IdentifierOr::Identifier(compile_identifier(inner)),
        Rule::basic_literal_expression => {
            let literal_rule = inner
                .into_inner()
                .next()
                .expect("missing literal for expression");
            ast::IdentifierOr::Value(value_parser(literal_rule))
        }
        _ => unreachable!(),
    }
}

enum Domain {
    Int(ast::IntDomain),
    Bool,
}

fn compile_domain(type_rule: Pair<'_, Rule>) -> Domain {
    assert_eq!(Rule::variable_type, type_rule.as_rule());

    let mut components = type_rule.into_inner();
    let first = components.next().expect("empty variable type");

    if first.as_rule() == Rule::basic_parameter_type {
        match first.as_str() {
            "int" => Domain::Int(ast::IntDomain::Unbounded),
            "bool" => Domain::Bool,
            _ => unreachable!(),
        }
    } else {
        let second = components.next().expect("missing upper bound");

        assert_eq!(Rule::int_literal, first.as_rule());
        assert_eq!(Rule::int_literal, second.as_rule());

        let lower = compile_int_literal(first);
        let upper = compile_int_literal(second);

        Domain::Int(ast::IntDomain::Interval { lower, upper })
    }
}

enum ParameterType {
    Int,
    Bool,
    ArrayOfInt(usize),
}

fn compile_parameter_declaration(
    parameter_declaration: Pair<'_, Rule>,
) -> Result<ast::ModelItem, FznError> {
    assert_eq!(Rule::parameter_declaration, parameter_declaration.as_rule());

    let mut components = parameter_declaration.into_inner();

    let type_rule = components.next().expect("missing parameter type");
    let identifier_rule = components.next().expect("missing parameter identifier");
    let value_rule = components.next().expect("missing parameter value");

    let parameter_type = compile_parameter_type(type_rule);
    let identifier = compile_identifier(identifier_rule);
    let value = compile_parameter_value(value_rule, parameter_type)?;

    Ok(ast::ModelItem::Parameter(ast::Parameter {
        identifier,
        value,
    }))
}

fn compile_parameter_type(type_rule: Pair<'_, Rule>) -> ParameterType {
    assert_eq!(Rule::parameter_type, type_rule.as_rule());

    let mut components = type_rule.into_inner();

    let first = components.next().expect("no value for parameter type");

    match first.as_rule() {
        Rule::basic_parameter_type => compile_basic_parameter_type(first),
        Rule::index_set => {
            let second = components.next().expect("no value for type of array value");
            compile_array_parameter_type(first, second)
        }
        _ => unreachable!(),
    }
}

fn compile_array_parameter_type(first: Pair<'_, Rule>, second: Pair<'_, Rule>) -> ParameterType {
    assert_eq!(Rule::index_set, first.as_rule());

    let num_elements = compile_index_set(first);

    match second.as_str() {
        "int" => ParameterType::ArrayOfInt(num_elements),
        _ => unreachable!(),
    }
}

fn compile_index_set(index_set: Pair<'_, Rule>) -> usize {
    assert_eq!(Rule::index_set, index_set.as_rule());

    index_set
        .into_inner()
        .next()
        .expect("no value for the number of elements in the parameter array")
        .as_str()
        .parse()
        .expect("index set literal not a valid usize")
}

fn compile_basic_parameter_type(basic_type_rule: Pair<'_, Rule>) -> ParameterType {
    assert_eq!(Rule::basic_parameter_type, basic_type_rule.as_rule());

    match basic_type_rule.as_str() {
        "int" => ParameterType::Int,
        "bool" => ParameterType::Bool,
        _ => unreachable!(),
    }
}

fn compile_parameter_value(
    value_rule: Pair<'_, Rule>,
    parameter_type: ParameterType,
) -> Result<ast::Value, FznError> {
    assert_eq!(Rule::parameter_expression, value_rule.as_rule());

    let mut basic_expressions = value_rule.into_inner();

    let value = match parameter_type {
        ParameterType::Int => {
            let basic_literal = basic_expressions.next().expect("missing literal");
            assert_eq!(Rule::basic_literal_expression, basic_literal.as_rule());

            let int_literal_rule = basic_literal.into_inner().next().expect("no int literal");

            let value = compile_int_literal(int_literal_rule);
            ast::Value::Int(value)
        }
        ParameterType::Bool => {
            let basic_literal = basic_expressions.next().expect("missing literal");
            assert_eq!(Rule::basic_literal_expression, basic_literal.as_rule());

            let bool_literal_rule = basic_literal.into_inner().next().expect("no bool literal");

            assert_eq!(Rule::bool_literal, bool_literal_rule.as_rule());
            ast::Value::Bool(bool_literal_rule.as_str().parse().expect("invalid bool"))
        }
        ParameterType::ArrayOfInt(num_elements) => {
            let array = basic_expressions
                .take(num_elements)
                .map(|basic_literal| {
                    assert_eq!(Rule::basic_literal_expression, basic_literal.as_rule());

                    let int_literal_rule =
                        basic_literal.into_inner().next().expect("no int literal");
                    compile_int_literal(int_literal_rule)
                })
                .collect::<Box<[_]>>();

            assert_eq!(
                array.len(),
                num_elements,
                "parameter array is does not match index set"
            );

            ast::Value::ArrayOfInt(array)
        }
    };

    Ok(value)
}

fn compile_int_literal(literal_rule: Pair<'_, Rule>) -> ast::Int {
    assert_eq!(Rule::int_literal, literal_rule.as_rule());
    literal_rule.as_str().parse().expect("invalid integer")
}

fn compile_bool_literal(literal_rule: Pair<'_, Rule>) -> bool {
    assert_eq!(Rule::bool_literal, literal_rule.as_rule());
    literal_rule.as_str().parse().expect("invalid boolean")
}

fn compile_identifier(identifier_rule: Pair<'_, Rule>) -> ast::Identifier {
    assert_eq!(Rule::identifier, identifier_rule.as_rule());
    ast::Identifier::from(identifier_rule.as_str())
}
