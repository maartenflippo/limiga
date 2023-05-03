use crate::{domains::Domain, variables::Variable};

use super::{PropagationResult, Propagator};

/// Create a propagator for the constraint `a != b`. Both variables 'a' and 'b' must have the same
/// type.
pub fn not_eq<VStore, Value, VX, VY>(a: VX, b: VY) -> Box<dyn Propagator<VStore>>
where
    Value: PartialEq + Clone,
    VX: Variable<VStore, Value = Value> + 'static,
    VY: Variable<VStore, Value = Value> + 'static,
{
    Box::new(NotEq { a, b })
}

struct NotEq<VX, VY> {
    a: VX,
    b: VY,
}

impl<VX, VY, VStore, Value> Propagator<VStore> for NotEq<VX, VY>
where
    Value: PartialEq + Clone,
    VX: Variable<VStore, Value = Value>,
    VY: Variable<VStore, Value = Value>,
{
    fn propagate(&mut self, store: &mut VStore) -> PropagationResult {
        propagate_one_direction(&self.a, &self.b, store)?;
        propagate_one_direction(&self.b, &self.a, store)?;

        PropagationResult::Consistent
    }
}

fn propagate_one_direction<VX, VY, VStore, Value>(
    a: &VX,
    b: &VY,
    store: &mut VStore,
) -> PropagationResult
where
    Value: PartialEq + Clone,
    VX: Variable<VStore, Value = Value>,
    VY: Variable<VStore, Value = Value>,
{
    let value_to_remove = {
        let a_dom = a.domain(&*store);
        a_dom.fixed_value().cloned()
    };

    if let Some(value) = value_to_remove {
        let b_dom = b.domain_mut(&mut *store);
        b_dom.remove(&value).into()
    } else {
        PropagationResult::Consistent
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_values_are_removed_when_other_is_fixed() {
        let mut doms = vec![vec![1, 2, 3], vec![1]];
        let mut propagator = not_eq(0, 1);

        propagator.propagate(&mut doms);

        assert_eq!(vec![vec![2, 3], vec![1]], doms);
    }

    #[test]
    fn test_no_values_are_removed_if_no_fixed_domains() {
        let doms_orig = vec![vec![1, 2, 3], vec![1, 2]];
        let mut doms = doms_orig.clone();
        let mut propagator = not_eq(0, 1);

        propagator.propagate(&mut doms);

        assert_eq!(doms_orig, doms);
    }

    impl Domain for Vec<i64> {
        type Value = i64;

        fn fixed_value(&self) -> Option<&Self::Value> {
            if self.len() == 1 {
                Some(&self[0])
            } else {
                None
            }
        }

        fn min(&self) -> &Self::Value {
            todo!()
        }

        fn max(&self) -> &Self::Value {
            todo!()
        }

        fn size(&self) -> usize {
            todo!()
        }

        fn remove(&mut self, value: &Self::Value) -> bool {
            let idx = self.iter().position(|v| v == value);

            if let Some(idx) = idx {
                self.remove(idx);
            }

            self.is_empty()
        }

        fn set_max(&mut self, _value: &Self::Value) {
            todo!()
        }

        fn set_min(&mut self, _value: &Self::Value) {
            todo!()
        }
    }

    impl Variable<Vec<Vec<i64>>> for usize {
        type Value = i64;

        type Dom = Vec<i64>;

        fn domain<'store>(&self, store: &'store Vec<Vec<i64>>) -> &'store Self::Dom {
            &store[*self]
        }

        fn domain_mut<'store>(&self, store: &'store mut Vec<Vec<i64>>) -> &'store mut Self::Dom {
            &mut store[*self]
        }
    }
}
