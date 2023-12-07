use pest::iterators::{Pair, Pairs};

use crate::{ast, FznError, Rule};

/// Constrains `\sum variables_i * weights_i != rhs`.
#[derive(Debug, PartialEq, Eq)]
pub struct IntLinNe {
    pub variables: ast::IdentifierOr<Box<[ast::IdentifierOr<ast::Int>]>>,
    pub weights: ast::IdentifierOr<Box<[ast::IdentifierOr<ast::Int>]>>,
    pub rhs: ast::IdentifierOr<ast::Int>,
}

impl IntLinNe {
    pub fn parse(mut arguments: Pairs<'_, Rule>) -> Result<IntLinNe, FznError> {
        let weights_rule = arguments.next().expect("missing weights for int_lin_ne");
        let variables_rule = arguments.next().expect("missing variables for int_lin_ne");
        let rhs = arguments.next().expect("missing rhs for int_lin_ne");

        let weights =
            compile_array_argument::<ast::Int>(weights_rule, compile_int_basic_expression)?;
        let variables =
            compile_array_argument::<ast::Int>(variables_rule, compile_int_basic_expression)?;
        let rhs = compile_int_argument(rhs)?;

        Ok(IntLinNe {
            variables,
            weights,
            rhs,
        })
    }
}

fn compile_int_argument(rule: Pair<'_, Rule>) -> Result<ast::IdentifierOr<ast::Int>, FznError> {
    assert_eq!(Rule::expression, rule.as_rule());

    let basic_expression = rule.into_inner().next().expect("expression is empty");
    assert_eq!(Rule::basic_expression, basic_expression.as_rule());

    compile_int_basic_expression(basic_expression)
}

fn compile_int_basic_expression(
    rule: Pair<'_, Rule>,
) -> Result<ast::IdentifierOr<ast::Int>, FznError> {
    assert_eq!(Rule::basic_expression, rule.as_rule());

    let inner = rule.into_inner().next().expect("basic expression is empty");
    match inner.as_rule() {
        Rule::identifier => Ok(ast::IdentifierOr::Identifier(inner.as_str().into())),
        Rule::basic_literal_expression => {
            let int_literal = inner
                .into_inner()
                .next()
                .expect("basic ltieral expression is empty");
            assert_eq!(Rule::int_literal, int_literal.as_rule());

            let value = int_literal
                .as_str()
                .parse::<ast::Int>()
                .expect("invalid integer");

            Ok(ast::IdentifierOr::Value(value))
        }
        _ => unreachable!(),
    }
}

fn compile_array_argument<T>(
    rule: Pair<'_, Rule>,
    element_parser: impl Fn(Pair<'_, Rule>) -> Result<ast::IdentifierOr<T>, FznError>,
) -> Result<ast::IdentifierOr<Box<[ast::IdentifierOr<T>]>>, FznError> {
    assert_eq!(Rule::expression, rule.as_rule());

    let expression = rule.into_inner().next().expect("expression is empty");

    match expression.as_rule() {
        Rule::array_literal => {
            let array = expression
                .into_inner()
                .map(element_parser)
                .collect::<Result<Box<[ast::IdentifierOr<T>]>, FznError>>()?;

            Ok(ast::IdentifierOr::Value(array))
        }

        Rule::basic_expression => {
            let basic_expression = expression
                .into_inner()
                .next()
                .expect("basic expression is empty");

            assert_eq!(Rule::identifier, basic_expression.as_rule());

            Ok(ast::IdentifierOr::Identifier(
                basic_expression.as_str().into(),
            ))
        }

        _ => unreachable!(),
    }
}
