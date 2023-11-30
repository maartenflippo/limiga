use pest::iterators::Pair;

use crate::{ast::Annotation, Rule};

pub fn compile_output_array_annotation(args: Pair<'_, Rule>) -> Annotation {
    assert_eq!(Rule::annotation_expression, args.as_rule());

    let expression = args
        .into_inner()
        .next()
        .expect("missing index set for output_array annotation");
    assert_eq!(Rule::basic_literal_expression, expression.as_rule());

    let literal_expression = expression
        .into_inner()
        .next()
        .expect("not a literal expression");
    assert_eq!(Rule::set_literal, literal_expression.as_rule());

    let mut bounds = literal_expression.into_inner();
    let start_idx_rule = bounds.next().unwrap();
    let end_idx_rule = bounds.next().unwrap();

    assert_eq!(Rule::int_literal, start_idx_rule.as_rule());
    assert_eq!(Rule::int_literal, end_idx_rule.as_rule());

    let start_idx = start_idx_rule
        .as_str()
        .parse::<usize>()
        .expect("invalid index");
    let end_idx = end_idx_rule
        .as_str()
        .parse::<usize>()
        .expect("invalid index");

    assert_eq!(1, start_idx, "output_array index sets should start at 1");

    Annotation::Output([end_idx].into())
}
