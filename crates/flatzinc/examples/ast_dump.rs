use std::fs::File;

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("provide a file path to the flatzinc file");

    let file = File::open(path).expect("failed to open file");

    for model_item in limiga_flatzinc::parse(file) {
        match model_item {
            Ok(model_item) => println!("{model_item:#?}"),
            Err(e) => {
                eprintln!("{}", e);
                break;
            }
        }
    }
}
