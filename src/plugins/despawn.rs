use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use bevy::{
    app::Plugin,
    ecs::{
        component::{self, Component, ComponentHooks, ComponentId, StorageType},
        system::Resource,
        world::DeferredWorld,
    },
    input::{keyboard::KeyCode, mouse::MouseMotion},
    math::Vec2,
    prelude::*,
    state::commands,
};
use chrono::{DateTime, TimeDelta, Utc};
use serde::de;

use crate::plugins::timer::{Timer, TimerCollection, TimerElapsedEvent, TimerLink};

use crate::stores::pause::PauseState;

#[derive(Reflect, Clone, Debug, Component)]
#[reflect(Component)]
#[component(on_add = resolve)]
pub struct Despawn {
    recursive: bool,
    duration: Option<Duration>,
    timer: Option<TimerLink>,
}

impl Despawn {
    pub fn now() -> Self {
        Self {
            recursive: false,
            duration: None,
            timer: None,
        }
    }

    pub fn after(duration: Duration) -> Self {
        Self {
            recursive: false,
            duration: Some(duration),
            timer: None,
        }
    }

    pub fn recursive(mut self) -> Self {
        self.recursive = true;
        self
    }
}

fn resolve(mut world: DeferredWorld<'_>, entity: Entity, _: ComponentId) {
    let mut despawn = world.get_mut::<Despawn>(entity).unwrap();

    let Some(duration) = despawn.duration.take() else {
        if despawn.recursive {
            world.commands().entity(entity).despawn_recursive();
        }

        world.commands().entity(entity).despawn();

        return;
    };

    let mut timers = world.resource_mut::<TimerCollection>();

    let handle = timers.add(Timer::new(duration).with_subscriber(entity));

    world.get_mut::<Despawn>(entity).unwrap().timer = Some(handle);
}

pub struct DespawnPlugin;

impl Plugin for DespawnPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Despawn>()
            .add_observer(handle_timer_elapsed);
    }
}

fn handle_timer_elapsed(
    trigger: Trigger<TimerElapsedEvent>,
    mut entity_q: Query<&mut Despawn>,
    mut commands: Commands,
    mut timers: ResMut<TimerCollection>,
) {
    let entity = trigger.entity();

    let Ok(despawn) = entity_q.get_mut(entity) else {
        return;
    };

    let Some(timer) = &despawn.timer else {
        return;
    };

    if *timer != trigger.event().0 {
        return;
    }

    timers.remove(timer);

    if despawn.recursive {
        commands.entity(entity).despawn_recursive();

        return;
    }

    commands.entity(entity).despawn();
}
