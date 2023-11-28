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

    #[error("syntax error")]
    Syntax(#[from] pest::error::Error<Rule>),
}

/// Parse a flatzinc source into an AST. The parser operates under the assumption that each model
/// item is on a separate line, which matches how the minizinc toolchain produces flatzinc.
pub fn parse(source: impl Read) -> impl Iterator<Item = Result<ast::ModelItem, FznError>> {
    let reader = BufReader::new(source);

    reader
        .lines()
        .map::<Result<ast::ModelItem, FznError>, _>(|line| {
            let line = line?;
            let model_item = FlatZincParser::parse(Rule::model_item, line.as_str())?
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

        _ => unreachable!(),
    }
}

enum ParameterType {
    Int,
    Bool,
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
    let identifier = ast::Identifier::from(identifier_rule.as_str());
    let value = compile_parameter_value(value_rule, parameter_type)?;

    Ok(ast::ModelItem::Parameter(ast::Parameter {
        identifier,
        value,
    }))
}

fn compile_parameter_type(type_rule: Pair<'_, Rule>) -> ParameterType {
    assert_eq!(Rule::parameter_type, type_rule.as_rule());

    match type_rule.as_str() {
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
    let literal_rule = value_rule.into_inner().next().expect("missing literal");

    let value = match parameter_type {
        ParameterType::Int => {
            assert_eq!(Rule::int_literal, literal_rule.as_rule());
            ast::Value::Int(literal_rule.as_str().parse().expect("invalid integer"))
        }
        ParameterType::Bool => {
            assert_eq!(Rule::bool_literal, literal_rule.as_rule());
            ast::Value::Bool(literal_rule.as_str().parse().expect("invalid bool"))
        }
    };

    Ok(value)
}
