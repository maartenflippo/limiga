use limiga_flatzinc::ast;

#[test]
fn integer_variable_declaration() {
    let source = "var int: SomeVar;";

    let ast = limiga_flatzinc::parse(source.as_bytes())
        .next()
        .expect("empty source")
        .expect("invalid variable declaration");

    let expected = ast::ModelItem::Variable(ast::Variable {
        identifier: "SomeVar".into(),
        domain: ast::Domain::Int(ast::IntDomain::Unbounded),
    });

    assert_eq!(expected, ast);
}

#[test]
fn interval_integer_variable_declaration() {
    let source = "var 1..10: SomeVar;";

    let ast = limiga_flatzinc::parse(source.as_bytes())
        .next()
        .expect("empty source")
        .expect("invalid variable declaration");

    let expected = ast::ModelItem::Variable(ast::Variable {
        identifier: "SomeVar".into(),
        domain: ast::Domain::Int(ast::IntDomain::Interval {
            lower: 1,
            upper: 10,
        }),
    });

    assert_eq!(expected, ast);
}

#[test]
fn boolean_variable_declaration() {
    let source = "var bool: SomeVar;";

    let ast = limiga_flatzinc::parse(source.as_bytes())
        .next()
        .expect("empty source")
        .expect("invalid variable declaration");

    let expected = ast::ModelItem::Variable(ast::Variable {
        identifier: "SomeVar".into(),
        domain: ast::Domain::Bool,
    });

    assert_eq!(expected, ast);
}
