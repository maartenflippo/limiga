use limiga_flatzinc::{ast, constraints};

#[test]
fn int_lin_ne_declaration() {
    let source = "constraint int_lin_ne([param, -1], x, 0);";

    let model_item = limiga_flatzinc::parse(source.as_bytes())
        .next()
        .expect("empty source")
        .expect("invalid constraint declaration");

    let expected = ast::ModelItem::Constraint(ast::Constraint::IntLinNe(constraints::IntLinNe {
        variables: ast::IdentifierOr::Identifier("x".into()),
        weights: ast::IdentifierOr::Value(
            [
                ast::IdentifierOr::Identifier("param".into()),
                ast::IdentifierOr::Value(-1),
            ]
            .into(),
        ),
        rhs: ast::IdentifierOr::Value(0),
    }));

    assert_eq!(expected, model_item);
}
