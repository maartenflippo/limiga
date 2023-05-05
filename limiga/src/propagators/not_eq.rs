use crate::variables::Variable;

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
    let value_to_remove = a.fixed_value(store).cloned();

    if let Some(value) = value_to_remove {
        b.remove(store, &value).into()
    } else {
        PropagationResult::Consistent
    }
}

#[cfg(test)]
mod tests {
    use crate::domains::Domain;

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

        fn set_max(&mut self, _value: &Self::Value) -> bool {
            todo!()
        }

        fn set_min(&mut self, _value: &Self::Value) -> bool {
            todo!()
        }
    }

    impl Variable<Vec<Vec<i64>>> for usize {
        type Value = i64;

        fn min<'store>(&self, store: &'store Vec<Vec<i64>>) -> &'store Self::Value {
            let dom = &store[*self];
            <Vec<i64> as Domain>::min(&dom)
        }

        fn max<'store>(&self, store: &'store Vec<Vec<i64>>) -> &'store Self::Value {
            let dom = &store[*self];
            <Vec<i64> as Domain>::max(&dom)
        }

        fn fixed_value<'store>(&self, store: &'store Vec<Vec<i64>>) -> Option<&'store Self::Value> {
            let dom = &store[*self];
            <Vec<i64> as Domain>::fixed_value(&dom)
        }

        fn remove(&self, store: &mut Vec<Vec<i64>>, value: &Self::Value) -> bool {
            let dom = &mut store[*self];
            <Vec<i64> as Domain>::remove(dom, value)
        }

        fn set_min(&self, store: &mut Vec<Vec<i64>>, value: &Self::Value) -> bool {
            let dom = &mut store[*self];
            <Vec<i64> as Domain>::set_min(dom, value)
        }

        fn set_max(&self, store: &mut Vec<Vec<i64>>, value: &Self::Value) -> bool {
            let dom = &mut store[*self];
            <Vec<i64> as Domain>::set_max(dom, value)
        }
    }
}
