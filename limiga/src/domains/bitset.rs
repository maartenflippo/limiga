use bit_vec::BitVec;

use super::Domain;

/// A bit set domain stores the domain values in a bitset. For large domains, there is a large
/// memory footprint, but calling [`Domain::remove()`] is cheap.
pub struct BitSetDomain {
    offset: i64,

    lower_bound: i64,
    upper_bound: i64,

    size: usize,
    values: BitVec,
}

impl BitSetDomain {
    /// Create a new domain with a given lower and upper bound.
    pub fn new(lower_bound: i64, upper_bound: i64) -> Self {
        let size = upper_bound.abs_diff(lower_bound) as usize + 1;

        BitSetDomain {
            offset: lower_bound,
            lower_bound,
            upper_bound,
            size,
            values: BitVec::from_elem(size, true),
        }
    }
}

impl Domain for BitSetDomain {
    type Value = i64;

    fn fixed_value(&self) -> Option<Self::Value> {
        if self.size() == 1 {
            Some(self.min())
        } else {
            None
        }
    }

    fn min(&self) -> Self::Value {
        self.lower_bound
    }

    fn max(&self) -> Self::Value {
        self.upper_bound
    }

    fn size(&self) -> usize {
        self.size
    }

    fn remove(&mut self, value: &Self::Value) -> bool {
        let mut bit_idx = value.abs_diff(self.offset) as usize;
        let is_present = self.values[bit_idx];

        self.values.set(bit_idx, false);
        self.size -= usize::from(is_present);

        if *value == self.min() {
            while !self.values[bit_idx] {
                self.lower_bound += 1;
                bit_idx += 1;
            }
        } else if *value == self.max() {
            while !self.values[bit_idx] {
                self.upper_bound -= 1;
                bit_idx -= 1;
            }
        }

        true
    }

    fn set_max(&mut self, value: &Self::Value) -> bool {
        self.size -= self.upper_bound.abs_diff(*value) as usize;
        self.upper_bound = *value;
        true
    }

    fn set_min(&mut self, value: &Self::Value) -> bool {
        self.size -= value.abs_diff(self.lower_bound) as usize;
        self.lower_bound = *value;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_bitset_domain_has_correct_properties() {
        let domain = BitSetDomain::new(1, 4);

        assert_eq!(1, domain.min());
        assert_eq!(4, domain.max());
        assert_eq!(4, domain.size());
        assert_eq!(None, domain.fixed_value());
    }

    #[test]
    fn removing_values_decreases_size() {
        let mut domain = BitSetDomain::new(1, 4);

        domain.remove(&2);

        assert_eq!(1, domain.min());
        assert_eq!(4, domain.max());
        assert_eq!(3, domain.size());
    }

    #[test]
    fn removing_lower_bound_updates_min() {
        let mut domain = BitSetDomain::new(1, 4);

        domain.remove(&2);
        domain.remove(&1);

        assert_eq!(3, domain.min());
        assert_eq!(4, domain.max());
        assert_eq!(2, domain.size());
    }

    #[test]
    fn removing_upper_bound_updates_min() {
        let mut domain = BitSetDomain::new(1, 4);

        domain.remove(&3);
        domain.remove(&4);

        assert_eq!(1, domain.min());
        assert_eq!(2, domain.max());
        assert_eq!(2, domain.size());
    }

    #[test]
    fn set_lower_bound_moves_min() {
        let mut domain = BitSetDomain::new(1, 4);

        domain.set_min(&3);

        assert_eq!(3, domain.min());
        assert_eq!(4, domain.max());
        assert_eq!(2, domain.size());
    }

    #[test]
    fn set_upper_bound_moves_max() {
        let mut domain = BitSetDomain::new(1, 4);

        domain.set_max(&2);

        assert_eq!(1, domain.min());
        assert_eq!(2, domain.max());
        assert_eq!(2, domain.size());
    }
}
