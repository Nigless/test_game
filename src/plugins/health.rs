use bevy::{
    ecs::{component::ComponentId, reflect, world::DeferredWorld},
    prelude::*,
    state::commands,
};

#[derive(Event)]
pub struct DeadEvent;

#[derive(Event)]
pub struct DamageInvokedEvent(pub u16);

#[derive(Component, Reflect)]
#[component(on_add = resolve)]
#[reflect(Component)]
pub struct Health {
    points: u16,
}

impl Health {
    pub fn new(points: u16) -> Self {
        Self { points }
    }

    fn apply_damage(&mut self, amount: u16) {
        self.points = self.points.saturating_sub(amount);
    }
}

fn resolve(mut world: DeferredWorld<'_>, entity: Entity, _component_id: ComponentId) {
    world.commands().entity(entity).observe(handle_damage);
}

fn handle_damage(
    trigger: Trigger<DamageInvokedEvent>,
    mut entity_q: Query<&mut Health>,
    mut commands: Commands,
) {
    let entity = trigger.entity();

    let Ok(mut health) = entity_q.get_mut(entity) else {
        return;
    };

    if health.points == 0 {
        return;
    }

    health.apply_damage(trigger.0);

    if health.points == 0 {
        commands.entity(entity).trigger(DeadEvent);
    }
}

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Health>();
    }
}
