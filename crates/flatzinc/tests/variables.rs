use limiga_flatzinc::ast;

#[test]
fn integer_variable_declaration() {
    let source = "var int: SomeVar;";

    let ast = limiga_flatzinc::parse(source.as_bytes())
        .next()
        .expect("empty source")
        .expect("invalid variable declaration");

    let expected = ast::ModelItem::Variable(ast::Variable::IntVariable(ast::SingleVariable {
        identifier: "SomeVar".into(),
        domain: ast::IntDomain::Unbounded,
    }));

    assert_eq!(expected, ast);
}

#[test]
fn interval_integer_variable_declaration() {
    let source = "var 1..10: SomeVar;";

    let ast = limiga_flatzinc::parse(source.as_bytes())
        .next()
        .expect("empty source")
        .expect("invalid variable declaration");

    let expected = ast::ModelItem::Variable(ast::Variable::IntVariable(ast::SingleVariable {
        identifier: "SomeVar".into(),
        domain: ast::IntDomain::Interval {
            lower: 1,
            upper: 10,
        },
    }));

    assert_eq!(expected, ast);
}

#[test]
fn array_of_integer_variable_declaration() {
    let source = "array [1..3] of var int: SomeArray = [SomeVar1, SomeVar2, 2];";

    let ast = limiga_flatzinc::parse(source.as_bytes())
        .next()
        .expect("empty source")
        .expect("invalid variable declaration");

    let expected =
        ast::ModelItem::Variable(ast::Variable::ArrayOfIntVariable(ast::VariableArray {
            identifier: "SomeArray".into(),
            variables: [
                ast::IdentifierOr::Identifier("SomeVar1".into()),
                ast::IdentifierOr::Identifier("SomeVar2".into()),
                ast::IdentifierOr::Value(2),
            ]
            .into(),
        }));

    assert_eq!(expected, ast);
}

#[test]
fn boolean_variable_declaration() {
    let source = "var bool: SomeVar;";

    let ast = limiga_flatzinc::parse(source.as_bytes())
        .next()
        .expect("empty source")
        .expect("invalid variable declaration");

    let expected = ast::ModelItem::Variable(ast::Variable::BoolVariable(ast::SingleVariable {
        identifier: "SomeVar".into(),
        domain: (),
    }));

    assert_eq!(expected, ast);
}

#[test]
fn array_of_boolean_variable_declaration() {
    let source = "array [1..3] of var bool: SomeArray = [SomeVar1, SomeVar2, false];";

    let ast = limiga_flatzinc::parse(source.as_bytes())
        .next()
        .expect("empty source")
        .expect("invalid variable declaration");

    let expected =
        ast::ModelItem::Variable(ast::Variable::ArrayOfBoolVariable(ast::VariableArray {
            identifier: "SomeArray".into(),
            variables: [
                ast::IdentifierOr::Identifier("SomeVar1".into()),
                ast::IdentifierOr::Identifier("SomeVar2".into()),
                ast::IdentifierOr::Value(false),
            ]
            .into(),
        }));

    assert_eq!(expected, ast);
}
