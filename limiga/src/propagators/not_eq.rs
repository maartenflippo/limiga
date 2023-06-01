use crate::{variables::Variable, Register};

use super::{PropagationResult, Propagator, RegistrationContext};

/// Create a propagator for the constraint `a != b`. Both variables 'a' and 'b' must have the same
/// type.
pub fn not_eq<VStore, Value, VX, VY, Registrar>(
    a: VX,
    b: VY,
) -> Box<dyn Propagator<VStore, Registrar>>
where
    Value: PartialEq + Clone,
    VX: Variable<VStore, Value = Value> + Register<Registrar> + 'static,
    VY: Variable<VStore, Value = Value> + Register<Registrar> + 'static,
    Registrar: RegistrationContext<VX::Dom> + RegistrationContext<VY::Dom>,
{
    Box::new(NotEq { a, b })
}

struct NotEq<VX, VY> {
    a: VX,
    b: VY,
}

impl<VX, VY, VStore, Value, Registrar> Propagator<VStore, Registrar> for NotEq<VX, VY>
where
    Value: PartialEq + Clone,
    VX: Variable<VStore, Value = Value> + Register<Registrar>,
    VY: Variable<VStore, Value = Value> + Register<Registrar>,
    Registrar: RegistrationContext<VX::Dom> + RegistrationContext<VY::Dom>,
{
    fn initialize(&mut self, ctx: &mut Registrar) {
        self.a.register(&mut *ctx);
        self.b.register(&mut *ctx);
    }

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
    let value_to_remove = a.fixed_value(store);

    if let Some(value) = value_to_remove {
        b.remove(store, &value).into()
    } else {
        PropagationResult::Consistent
    }
}

#[cfg(test)]
mod tests {
    use crate::domains::{Domain, DomainId, DomainMut};

    use super::*;

    #[test]
    fn test_values_are_removed_when_other_is_fixed() {
        let mut doms = vec![vec![1, 2, 3], vec![1]];
        let mut propagator: Box<dyn Propagator<_, TestRegistrar>> = not_eq(0, 1);

        propagator.propagate(&mut doms);

        assert_eq!(vec![vec![2, 3], vec![1]], doms);
    }

    #[test]
    fn test_no_values_are_removed_if_no_fixed_domains() {
        let doms_orig = vec![vec![1, 2, 3], vec![1, 2]];
        let mut doms = doms_orig.clone();
        let mut propagator: Box<dyn Propagator<_, TestRegistrar>> = not_eq(0, 1);

        propagator.propagate(&mut doms);

        assert_eq!(doms_orig, doms);
    }

    impl Domain for Vec<i64> {
        type Value = i64;

        fn fixed_value(&self) -> Option<Self::Value> {
            if self.len() == 1 {
                Some(self[0])
            } else {
                None
            }
        }

        fn min(&self) -> Self::Value {
            todo!()
        }

        fn max(&self) -> Self::Value {
            todo!()
        }

        fn size(&self) -> usize {
            todo!()
        }
    }

    impl DomainMut for Vec<i64> {
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

        fn fix(&mut self, _value: &Self::Value) -> bool {
            todo!()
        }
    }

    impl Variable<Vec<Vec<i64>>> for usize {
        type Value = i64;
        type Dom = Vec<i64>;

        fn min(&self, store: &Vec<Vec<i64>>) -> Self::Value {
            let dom = &store[*self];
            <Vec<i64> as Domain>::min(&dom)
        }

        fn max(&self, store: &Vec<Vec<i64>>) -> Self::Value {
            let dom = &store[*self];
            <Vec<i64> as Domain>::max(&dom)
        }

        fn size(&self, store: &Vec<Vec<i64>>) -> usize {
            let dom = &store[*self];
            <Vec<i64> as Domain>::size(&dom)
        }

        fn fixed_value(&self, store: &Vec<Vec<i64>>) -> Option<Self::Value> {
            let dom = &store[*self];
            <Vec<i64> as Domain>::fixed_value(&dom)
        }

        fn remove(&self, store: &mut Vec<Vec<i64>>, value: &Self::Value) -> bool {
            let dom = &mut store[*self];
            <Vec<i64> as DomainMut>::remove(dom, value)
        }

        fn set_min(&self, store: &mut Vec<Vec<i64>>, value: &Self::Value) -> bool {
            let dom = &mut store[*self];
            <Vec<i64> as DomainMut>::set_min(dom, value)
        }

        fn set_max(&self, store: &mut Vec<Vec<i64>>, value: &Self::Value) -> bool {
            let dom = &mut store[*self];
            <Vec<i64> as DomainMut>::set_max(dom, value)
        }

        fn fix(&self, store: &mut Vec<Vec<i64>>, value: &Self::Value) -> bool {
            let dom = &mut store[*self];
            <Vec<i64> as DomainMut>::fix(dom, value)
        }
    }

    impl Register<TestRegistrar> for usize {
        fn register(&self, _: &mut TestRegistrar) {
            todo!()
        }
    }

    struct TestRegistrar;

    impl<Dom> RegistrationContext<Dom> for TestRegistrar {
        fn register(&mut self, _: DomainId<Dom>) {
            todo!()
        }
    }
}
