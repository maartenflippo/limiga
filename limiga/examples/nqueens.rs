use limiga::{
    domains::{BitSetDomain, Domains},
    propagators::not_eq,
    search::Branch,
    OffsetView, PropagatorRegistration, Register, SolveOutcome, Solver, Variable,
};

fn main() {
    let n = std::env::args()
        .nth(1)
        .expect("provide a value for 'n'")
        .parse::<usize>()
        .expect("'n' is not a valid number");

    let mut solver = Solver::default();

    let vars = (0..n)
        .map(|_| {
            solver.new_int_var(BitSetDomain::new(
                0,
                n.try_into().expect("could not convert 'n' to i64"),
            ))
        })
        .collect::<Vec<_>>();

    let diag_1 = vars
        .iter()
        .enumerate()
        .map(|(i, var)| OffsetView::new(*var, i as i64))
        .collect::<Vec<_>>();
    let diag_2 = vars
        .iter()
        .enumerate()
        .map(|(i, var)| OffsetView::new(*var, -(i as i64)))
        .collect::<Vec<_>>();

    all_different(&mut solver, &vars);
    all_different(&mut solver, &diag_1);
    all_different(&mut solver, &diag_2);

    let brancher = |store: &Domains| {
        if let Some(var) = vars.iter().min_by_key(|var| var.size(store)).cloned() {
            let var_b2 = var.clone();

            let val1 = var.min(store).clone();
            let val2 = var.min(store).clone();

            Some([
                Box::new(move |s: &mut Domains| {
                    let val = val1;
                    var.fix(s, &val);
                }) as Branch<Domains>,
                Box::new(move |store: &mut Domains| {
                    let val = val2;
                    var_b2.remove(store, &val);
                }) as Branch<Domains>,
            ])
        } else {
            None
        }
    };

    match solver.solve(brancher) {
        SolveOutcome::Satisfiable(mut solutions) => {
            while let Some(solution) = solutions.next() {
                let values = vars
                    .iter()
                    .map(|var| solution.value(*var))
                    .collect::<Vec<_>>();

                println!("SATISFIABLE");
                print_board(values);
            }
        }

        SolveOutcome::Unsatisfiable => println!("UNSATISFIABLE"),
    }
}

fn print_board(values: Vec<i64>) {
    let n = values.len();
    let row_separator = format!("{}+\n", "+---".repeat(values.len()));

    let board = values
        .into_iter()
        .map(|value| {
            let row = (0..n)
                .map(|col| {
                    if col == value.try_into().unwrap() {
                        "| * "
                    } else {
                        "|   "
                    }
                })
                .collect::<String>();

            format!("{row}|\n{row_separator}")
        })
        .collect::<String>();

    println!("{row_separator}{board}");
}

fn all_different<Var>(solver: &mut Solver, vars: &[Var])
where
    Var: Variable<Domains> + Register<PropagatorRegistration> + Clone + 'static,
    Var::Value: Clone,
{
    for i in 0..vars.len() {
        for j in i + 1..vars.len() {
            let a = vars[i].clone();
            let b = vars[j].clone();

            solver.post(not_eq(a, b));
        }
    }
}
