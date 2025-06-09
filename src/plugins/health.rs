use std::time::Duration;

use crate::plugins::timer::{self, Timer, TimerCollection, TimerElapsedEvent, TimerLink};
use bevy::{
    ecs::{component::ComponentId, world::DeferredWorld},
    log::tracing_subscriber::fmt::time,
    prelude::*,
};

#[derive(Event)]
pub struct DeadEvent;

#[derive(Event)]
pub struct DamageInvokedEvent(pub u16);

#[derive(Component, Reflect)]
#[reflect(Component)]
#[component(on_add = resolve, on_remove = clean_up)]
pub struct Health {
    initial: u16,
    current: u16,
    timer: Option<TimerLink>,
    #[reflect(skip_serializing)]
    duration: Option<Duration>,
}

impl Health {
    pub fn new(points: u16) -> Self {
        Self {
            initial: points,
            current: points,
            timer: None,
            duration: None,
        }
    }

    pub fn regenerate(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    fn apply_damage(&mut self, amount: u16) {
        self.current = self.current.saturating_sub(amount);
    }

    fn heal(&mut self, amount: u16) {
        if self.current >= self.initial {
            return;
        }

        self.current += self.initial.saturating_sub(self.current).min(amount);
    }

    fn is_dead(&self) -> bool {
        self.current == 0
    }
}

fn resolve(mut world: DeferredWorld<'_>, entity: Entity, _: ComponentId) {
    let Some(duration) = world
        .get_mut::<Health>(entity)
        .and_then(|mut h| h.duration.take())
    else {
        return;
    };

    let mut timers = world.resource_mut::<TimerCollection>();

    let handle = timers.add(Timer::new(duration).repeated().with_subscriber(entity));

    world.get_mut::<Health>(entity).unwrap().timer = Some(handle);
}

fn clean_up(mut world: DeferredWorld<'_>, entity: Entity, _: ComponentId) {
    let Some(timer) = world.get::<Health>(entity).and_then(|h| h.timer.clone()) else {
        return;
    };

    let mut timers = world.resource_mut::<TimerCollection>();

    timers.remove(&timer);
}

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Health>()
            .add_observer(handle_damage)
            .add_observer(handle_timer_elapsed);
    }
}

fn handle_damage(
    trigger: Trigger<DamageInvokedEvent>,
    mut entity_q: Query<&mut Health>,
    mut commands: Commands,
    mut timers: ResMut<TimerCollection>,
) {
    let entity = trigger.entity();

    let Ok(mut health) = entity_q.get_mut(entity) else {
        return;
    };

    if health.is_dead() {
        return;
    }

    let mut timer = health.timer.as_ref().map(|t| timers.get_mut(t).unwrap());

    timer.as_deref_mut().map(|t| t.reset());

    health.apply_damage(trigger.0);

    if health.is_dead() {
        commands.entity(entity).trigger(DeadEvent);
        timer.as_deref_mut().map(|t| t.stop());
    }
}

fn handle_timer_elapsed(trigger: Trigger<TimerElapsedEvent>, mut entity_q: Query<&mut Health>) {
    let Ok(mut health) = entity_q.get_mut(trigger.entity()) else {
        return;
    };

    let Some(timer) = &health.timer else {
        return;
    };

    if *timer != trigger.event().0 {
        return;
    }

    health.heal(1);
}
