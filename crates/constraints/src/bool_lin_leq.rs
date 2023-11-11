use limiga_core::{
    domains::DomainStore,
    integer::{BoundedInt, BoundedIntVar, Int, IntEvent},
    lit::Lit,
    propagation::{
        Context, DomainEvent, LitEvent, LocalId, Propagator, PropagatorFactory, PropagatorVar,
        Reason, VariableRegistrar, Watchable,
    },
    variable::Variable,
    Conflict,
};

pub struct LinearBoolFactory<VY> {
    pub x: Box<[Lit]>,
    pub y: VY,
}

impl<VY, Domains, Event> PropagatorFactory<Domains, Event> for LinearBoolFactory<VY>
where
    Event: DomainEvent<LitEvent, IntEvent>,
    VY: BoundedIntVar<Domains, Event> + Watchable<TypedEvent = IntEvent>,
    VY::Dom: BoundedInt,
    Domains: DomainStore<VY::Dom>,
{
    fn create(
        self,
        registrar: &mut VariableRegistrar<'_, Event>,
    ) -> Box<dyn Propagator<Domains, Event>> {
        let x: Box<[PropagatorVar<Lit>]> = self
            .x
            .iter()
            .copied()
            .enumerate()
            .map(|(i, x_i)| registrar.register(x_i, (i as u32).into(), LitEvent::FixedTrue))
            .collect();

        let y = registrar.register(self.y, (x.len() as u32).into(), IntEvent::UpperBound);

        Box::new(LinearBool { x, y })
    }
}

/// A propagator for the constraint `\sum x_i <= y`, where `x_i` are propositional literals and `y`
/// is an integer variable.
pub struct LinearBool<VY> {
    x: Box<[PropagatorVar<Lit>]>,
    y: PropagatorVar<VY>,
}

impl<VY, Domains, Event> Propagator<Domains, Event> for LinearBool<VY>
where
    Event: DomainEvent<LitEvent, IntEvent>,
    VY: BoundedIntVar<Domains, Event>,
    VY::Dom: BoundedInt,
    Domains: DomainStore<VY::Dom>,
{
    fn on_event(&mut self, variable: LocalId, event: Event) -> bool {
        let id_y = LocalId::from(self.x.len() as u32);

        if variable < id_y {
            assert!(event.is(LitEvent::FixedTrue));
        } else {
            assert!(variable == id_y);
            assert!(event.is(IntEvent::UpperBound));
        }

        true
    }

    fn propagate(&mut self, ctx: &mut Context<Domains, Event>) -> Result<(), Conflict> {
        // The lower bound of `self.y` is the number of literals fixed to true in `x`.
        let true_lits = self
            .x
            .iter()
            .filter(|&&x_i| ctx.value(x_i) == Some(true))
            .map(|&x_i| x_i.variable)
            .collect::<Box<[_]>>();
        let fixed_true_count = true_lits.len() as Int;

        self.y
            .set_min(ctx, fixed_true_count, Reason::from_literals(&true_lits))?;

        // If the number of fixed true literals equals the upper-bound of `self.y`, the remaining
        // literals can be fixed to false.
        //
        // Note: at this point the number of fixed true literals cannot exceed the upper bound of
        // `self.y` because the previous propagation would have taken the error path.
        if fixed_true_count == self.y.max(ctx) {
            for &x_i in self.x.iter() {
                if ctx.value(x_i).is_none() {
                    ctx.assign(x_i, false)
                        .expect("these assignments can all be made");
                }
            }
        }

        Ok(())
    }
}
