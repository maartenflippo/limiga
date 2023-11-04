use limiga_core::{
    lit::Lit,
    propagation::{
        DomainEvent, LitFixedEvent, LocalId, ProducesEvent, Propagator, PropagatorFactory,
        PropagatorVar, UpperBoundEvent, VariableRegistrar,
    },
};

pub struct LinearBoolFactory<VY> {
    x: Box<[Lit]>,
    y: VY,
}

impl<VY, Event> PropagatorFactory<Event> for LinearBoolFactory<VY>
where
    Event: DomainEvent<LitFixedEvent, UpperBoundEvent>,
    VY: ProducesEvent<UpperBoundEvent>,
{
    type Output = LinearBool<VY>;

    fn create(self, registrar: &mut VariableRegistrar<Event>) -> Self::Output {
        let x: Box<[PropagatorVar<Lit>]> = self
            .x
            .into_iter()
            .copied()
            .enumerate()
            .map(|(i, x_i)| registrar.register(x_i, LitFixedEvent, (i as u32).into()))
            .collect();

        let y = registrar.register(self.y, UpperBoundEvent, (x.len() as u32).into());

        LinearBool { x, y }
    }
}

/// A propagator for the constraint `\sum x_i <= y`, where `x_i` are propositional literals and `y`
/// is an integer variable.
pub struct LinearBool<VY> {
    x: Box<[PropagatorVar<Lit>]>,
    y: PropagatorVar<VY>,
}

impl<VY, Event> Propagator<Event> for LinearBool<VY>
where
    Event: DomainEvent<LitFixedEvent, UpperBoundEvent>,
{
    fn on_event(&mut self, variable: LocalId, event: Event) -> bool {
        let id_y = LocalId::from(self.x.len() as u32);

        if variable < id_y {
            assert!(event.is(LitFixedEvent));
        } else {
            assert!(variable == id_y);
            assert!(event.is(UpperBoundEvent));
        }

        true
    }
}
