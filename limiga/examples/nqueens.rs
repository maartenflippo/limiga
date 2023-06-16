use limiga::{
    domains::{BitSetDomain, Domains},
    propagators::not_eq,
    search::{partitioners::DomainMin, selectors::FirstFail, Search},
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
                1,
                n.try_into().expect("could not convert 'n' to i64"),
            ))
        })
        .collect::<Vec<_>>();

    let (diag_1, diag_2): (Vec<_>, Vec<_>) = vars
        .iter()
        .enumerate()
        .map(|(i, var)| {
            let i = i as i64;

            (OffsetView::new(*var, i), OffsetView::new(*var, -i))
        })
        .unzip();

    all_different(&mut solver, &vars);
    all_different(&mut solver, &diag_1);
    all_different(&mut solver, &diag_2);

    let brancher = Search::new(FirstFail::new(vars.clone()), DomainMin);
    match solver.solve(brancher) {
        SolveOutcome::Satisfiable(mut solutions) => {
            println!("SATISFIABLE");
            while let Some(solution) = solutions.next_solution() {
                let values = vars
                    .iter()
                    .map(|var| solution.value(*var))
                    .collect::<Vec<_>>();

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
                    if col == (value - 1).try_into().unwrap() {
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

/// For now, we do not have a dedicated propagator for this constraint. Therefore, we model it
/// using a decomposition into pairwaise inequalities.
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
