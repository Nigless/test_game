use std::{marker::PhantomData, thread::spawn};

use bevy::{
    ecs::component::{ComponentHooks, StorageType},
    prelude::*,
};

use crate::library::Spawnable;

#[derive(Reflect)]
#[reflect(Component)]
pub struct WithEntity<T: Spawnable + Send + Sync + 'static> {
    entity: Option<T>,
    child: bool,
}

impl<T: Spawnable + Send + Sync + 'static> WithEntity<T> {
    pub fn new(v: T) -> Self {
        Self {
            entity: Some(v),
            child: false,
        }
    }

    pub fn child(v: T) -> Self {
        Self {
            entity: Some(v),
            child: true,
        }
    }
}

impl<T: Spawnable + Default + Send + Sync + 'static> Default for WithEntity<T> {
    fn default() -> Self {
        Self {
            entity: Some(default()),
            child: false,
        }
    }
}

impl<T: Spawnable + Send + Sync + 'static> Component for WithEntity<T> {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_add(|mut world, entity, _| {
            let (child_bundle, child) = {
                let mut component = world.get_mut::<WithEntity<T>>(entity).unwrap();

                (component.entity.take(), component.child)
            };

            let mut commands = world.commands();

            commands.entity(entity).remove::<WithEntity<T>>();

            let Some(child_bundle) = child_bundle else {
                return;
            };

            let child_entity = child_bundle.spawn(&mut commands).id();

            if !child {
                return;
            }

            commands.entity(entity).add_child(child_entity);
        });
    }
}
