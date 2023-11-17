use crate::{
    domains::{Conflict, Domain, DomainFactory, EnqueueDomainLit},
    lit::Lit,
    propagation::Explanation,
    solver::ExtendClausalSolver,
};

use super::{BoundedInt, Int, IntEvent};

/// An integer domain defined by an upper and lower bound. This domain does not support removing
/// individual values. Only operations on the bounds of the domain are supported.
pub struct IntInterval {
    lower_bound: Int,
    upper_bound: Int,

    literals: Box<[Lit]>,
}

impl IntInterval {
    pub fn factory(lower_bound: Int, upper_bound: Int) -> IntIntervalFactory {
        IntIntervalFactory {
            lower_bound,
            upper_bound,
        }
    }

    #[inline]
    fn literal(&self, value: Int) -> Lit {
        let idx = value
            .abs_diff(self.lower_bound)
            .min(self.literals.len() as u32 - 1) as usize;

        self.literals[idx]
    }
}

pub struct IntIntervalFactory {
    lower_bound: Int,
    upper_bound: Int,
}

impl<Event> DomainFactory<Event> for IntIntervalFactory {
    type Domain = IntInterval;

    fn create(self, clausal_solver: &mut impl ExtendClausalSolver<Event>) -> Self::Domain {
        let lb_lits = clausal_solver
            .new_lits()
            .take(self.upper_bound.abs_diff(self.lower_bound) as usize + 2)
            .collect::<Box<[_]>>();
        let domain = IntInterval {
            lower_bound: self.lower_bound,
            upper_bound: self.upper_bound,
            literals: lb_lits,
        };

        // for all v in the domain: [x >= v] -> [x >= v - 1]
        for v in (self.lower_bound + 1)..=self.upper_bound {
            clausal_solver.add_clause([!domain.literal(v), domain.literal(v - 1)]);
        }

        // ![x >= upper_bound + 1]
        clausal_solver.add_clause([!domain.literal(self.upper_bound + 1)]);

        // [x >= lower_bound]
        clausal_solver.add_clause([domain.literal(self.lower_bound)]);

        domain
    }
}

impl Domain for IntInterval {
    type ProducedEvent = IntEvent;
}

impl BoundedInt for IntInterval {
    fn max(&self) -> Int {
        self.upper_bound
    }

    fn max_lit(&self) -> Lit {
        self.literal(self.upper_bound)
    }

    fn set_min(
        &mut self,
        bound: Int,
        explanation: Explanation,
        mut enqueue_lit: impl EnqueueDomainLit,
    ) -> Result<(), Conflict> {
        if bound > self.lower_bound {
            enqueue_lit.enqueue(self.literal(bound), explanation)?;
            self.lower_bound = bound;
        }

        assert!(self.lower_bound <= self.upper_bound);

        Ok(())
    }
}
