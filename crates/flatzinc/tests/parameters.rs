use limiga_flatzinc::ast;

#[test]
fn integer_parameter_declaration() {
    let source = "int: SomeParam = 5;";

    let ast = limiga_flatzinc::parse(source.as_bytes())
        .next()
        .expect("empty source")
        .expect("invalid parameter declaration");

    let expected = ast::ModelItem::Parameter(ast::Parameter {
        identifier: "SomeParam".into(),
        value: ast::Value::Int(5),
    });

    assert_eq!(expected, ast);
}

#[test]
fn boolean_parameter_declaration() {
    let source = "bool: SomeParam = false;";

    let ast = limiga_flatzinc::parse(source.as_bytes())
        .next()
        .expect("empty source")
        .expect("invalid parameter declaration");

    let expected = ast::ModelItem::Parameter(ast::Parameter {
        identifier: "SomeParam".into(),
        value: ast::Value::Bool(false),
    });

    assert_eq!(expected, ast);
}
