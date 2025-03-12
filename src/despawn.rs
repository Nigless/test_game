use std::time::{Duration, Instant};

use bevy::{
    app::Plugin,
    ecs::{
        component::{Component, ComponentHooks, StorageType},
        system::Resource,
    },
    input::{keyboard::KeyCode, mouse::MouseMotion},
    math::Vec2,
    prelude::*,
    state::commands,
};
use serde::{Deserialize, Serialize};

use crate::library::Spawnable;

#[derive(Reflect, Serialize, Deserialize, Clone)]
#[reflect(Component)]
pub struct Despawn {
    recursive: bool,
    timeout: Option<Duration>,

    #[serde(skip_deserializing, skip_serializing)]
    crated_at: Option<Instant>,
}

impl Despawn {
    pub fn new() -> Self {
        Self {
            recursive: false,
            timeout: None,
            crated_at: None,
        }
    }

    pub fn recursive(mut self) -> Self {
        self.recursive = true;
        self
    }

    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(duration);
        self
    }

    pub fn is_time_up(&self) -> bool {
        let Some(crated_at) = self.crated_at else {
            return true;
        };

        let Some(timeout) = self.timeout else {
            return true;
        };

        crated_at.elapsed() > timeout
    }
}

impl Component for Despawn {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_add(|mut world, entity, _| {
            let despawn = world.get::<Despawn>(entity).cloned().unwrap();

            if despawn.timeout.is_some() {
                let mut despawn = world.get_mut::<Despawn>(entity).unwrap();

                despawn.crated_at = Some(Instant::now());

                return;
            }

            let mut w = world.commands();
            let mut commands = w.entity(entity);

            if despawn.recursive {
                commands.despawn_recursive();
                return;
            }
            commands.despawn();
        });
    }
}

#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub struct DespawnSystems;

pub struct DespawnPlugin;

impl Plugin for DespawnPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Despawn>()
            .add_systems(FixedPreUpdate, update.in_set(DespawnSystems));
    }
}

fn update(mut commands: Commands, entity_q: Query<(Entity, &Despawn)>) {
    for (entity, despawn) in entity_q.iter() {
        if !despawn.is_time_up() {
            return;
        }

        if despawn.recursive {
            commands.entity(entity).despawn_recursive();
            return;
        }
        commands.entity(entity).despawn();
    }
}
