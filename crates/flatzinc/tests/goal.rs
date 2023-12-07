use limiga_flatzinc::ast;

#[test]
fn solve_satisfy() {
    let source = "solve satisfy;";

    let model_item = limiga_flatzinc::parse(source.as_bytes())
        .next()
        .expect("empty source")
        .expect("invalid model item");

    let expected = ast::ModelItem::Goal(ast::Goal::Satisfy);

    assert_eq!(expected, model_item);
}
