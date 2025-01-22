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

#[derive(Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Despawn {
    recursive: bool,
}

impl Despawn {
    pub fn recursive() -> Self {
        Self { recursive: true }
    }
}

impl Component for Despawn {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_add(|mut world, entity, _component_id| {
            let recursive = world.get_mut::<Despawn>(entity).unwrap().recursive;

            let mut w = world.commands();
            let mut commands = w.entity(entity);

            if recursive {
                commands.despawn_recursive();
                return;
            }
            commands.despawn();
        });
    }
}

pub struct DespawnPlugin;

impl Plugin for DespawnPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Despawn>();
    }
}
