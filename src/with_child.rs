use std::{marker::PhantomData, thread::spawn};

use bevy::{
    ecs::component::{ComponentHooks, StorageType},
    prelude::*,
};

use crate::library::Spawnable;

#[derive(Reflect)]
#[reflect(Component)]
pub struct WithChild<T: Spawnable + Send + Sync + 'static> {
    child: Option<T>,
}

impl<T: Spawnable + Send + Sync + 'static> WithChild<T> {
    pub fn new(v: T) -> Self {
        Self { child: Some(v) }
    }
}

impl<T: Spawnable + Default + Send + Sync + 'static> Default for WithChild<T> {
    fn default() -> Self {
        Self {
            child: Some(default()),
        }
    }
}

impl<T: Spawnable + Send + Sync + 'static> Component for WithChild<T> {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_add(|mut world, entity, _| {
            let child = world.get_mut::<WithChild<T>>(entity).unwrap().child.take();

            let mut commands = world.commands();

            commands.entity(entity).remove::<WithChild<T>>();

            let Some(child) = child else {
                return;
            };

            let child_entity = child.spawn(&mut commands);

            commands.entity(entity).add_child(child_entity);
        });
    }
}
