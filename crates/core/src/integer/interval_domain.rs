use crate::{domains::Domain, Conflict};

use super::{BoundedInt, Int, IntEvent};

/// An integer domain defined by an upper and lower bound. This domain does not support removing
/// individual values. Only operations on the bounds of the domain are supported.
pub struct IntInterval {
    lower_bound: Int,
    upper_bound: Int,
}

impl IntInterval {
    pub fn new(lower_bound: Int, upper_bound: Int) -> IntInterval {
        IntInterval {
            lower_bound,
            upper_bound,
        }
    }
}

impl Domain for IntInterval {
    type ProducedEvent = IntEvent;
}

impl BoundedInt for IntInterval {
    fn max(&self) -> Int {
        self.upper_bound
    }

    fn set_min(&mut self, bound: Int) -> Result<(), Conflict> {
        if bound > self.lower_bound {
            self.lower_bound = bound;
        }

        if self.lower_bound <= self.upper_bound {
            Ok(())
        } else {
            Err(Conflict)
        }
    }
}
