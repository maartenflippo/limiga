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

#[test]
fn array_of_integer_parameter_declaration() {
    let source = "array [1..5] of int: SomeParam = [1, 2, 3, 4, 5];";

    let ast = limiga_flatzinc::parse(source.as_bytes())
        .next()
        .expect("empty source")
        .expect("invalid parameter declaration");

    let expected = ast::ModelItem::Parameter(ast::Parameter {
        identifier: "SomeParam".into(),
        value: ast::Value::ArrayOfInt([1, 2, 3, 4, 5].into()),
    });

    assert_eq!(expected, ast);
}
