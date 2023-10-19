use std::{fs::File, num::NonZeroI32, path::PathBuf};

use limiga_dimacs::DimacsSink;

macro_rules! cnf_instance {
    ($name:ident) => {
        #[test]
        fn $name() {
            let file = PathBuf::from(format!(
                "{}/instances/cnf/{}.cnf",
                env!("CARGO_MANIFEST_DIR"),
                stringify!($name),
            ));

            let result = limiga::run_solver(&file).expect("failed to run solver");

            let instance = limiga_dimacs::parse_cnf(
                File::open(file).expect("could not open instance file for checking"),
                |_| SatInstance::default(),
            )
            .expect("valid dimacs");

            if let Some(assignment) = result {
                instance.assert_satisfied(assignment);
            }
        }
    };
}

cnf_instance!(add4);
cnf_instance!(add8);
cnf_instance!(add16);
cnf_instance!(add32);
cnf_instance!(add64);
cnf_instance!(add128);
cnf_instance!(block0);
cnf_instance!(elimclash);
cnf_instance!(elimredundant);
cnf_instance!(empty);
cnf_instance!(factor2708413neg);
cnf_instance!(factor2708413pos);
cnf_instance!(trivially_false);
cnf_instance!(full1);
cnf_instance!(full2);
cnf_instance!(full3);
cnf_instance!(full4);
cnf_instance!(full5);
cnf_instance!(full6);
cnf_instance!(full7);
cnf_instance!(ph2);
cnf_instance!(ph3);
cnf_instance!(ph4);
cnf_instance!(ph5);
cnf_instance!(ph6);
cnf_instance!(prime4);
cnf_instance!(prime9);
cnf_instance!(prime25);
cnf_instance!(prime49);
cnf_instance!(prime121);
cnf_instance!(prime169);
cnf_instance!(prime289);
cnf_instance!(prime361);
cnf_instance!(prime529);
cnf_instance!(prime841);
cnf_instance!(prime961);
cnf_instance!(prime1369);
cnf_instance!(prime1681);
cnf_instance!(prime2209);
cnf_instance!(prime1849);
cnf_instance!(prime65537);
// cnf_instance!(prime4294967297);
cnf_instance!(regr000);
cnf_instance!(sat0);
cnf_instance!(sat1);
cnf_instance!(sat2);
cnf_instance!(sat3);
cnf_instance!(sat4);
cnf_instance!(sat5);
cnf_instance!(sat6);
cnf_instance!(sat7);
cnf_instance!(sat8);
cnf_instance!(sat9);
cnf_instance!(sat10);
cnf_instance!(sat11);
cnf_instance!(sat12);
cnf_instance!(sat13);
cnf_instance!(sqrt2809);
cnf_instance!(sqrt3481);
cnf_instance!(sqrt3721);
cnf_instance!(sqrt4489);
cnf_instance!(sqrt5041);
cnf_instance!(sqrt5329);
cnf_instance!(sqrt6241);
cnf_instance!(sqrt6889);
cnf_instance!(sqrt7921);
cnf_instance!(sqrt9409);
cnf_instance!(sqrt10201);
cnf_instance!(sqrt10609);
cnf_instance!(sqrt11449);
cnf_instance!(sqrt11881);
cnf_instance!(sqrt12769);
cnf_instance!(sqrt16129);
cnf_instance!(sqrt63001);
cnf_instance!(sqrt259081);
cnf_instance!(sqrt1042441);
cnf_instance!(sub0);
cnf_instance!(unit0);
cnf_instance!(unit1);
cnf_instance!(unit2);
cnf_instance!(unit3);
cnf_instance!(unit4);
cnf_instance!(unit5);
cnf_instance!(unit6);
cnf_instance!(unit7);

#[derive(Default)]
struct SatInstance {
    clauses: Vec<Box<[NonZeroI32]>>,
}

impl DimacsSink for SatInstance {
    fn add_clause(&mut self, clause: &[NonZeroI32]) {
        self.clauses.push(clause.into());
    }
}

impl SatInstance {
    fn assert_satisfied(&self, assignment: limiga::Assignment) {
        for clause in &self.clauses {
            if !clause.iter().any(|&lit| assignment.value(lit)) {
                panic!("unsatisfied clause");
            }
        }
    }
}
