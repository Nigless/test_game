use bevy::{
    ecs::component::{ComponentHooks, StorageType},
    prelude::*,
};

pub struct WithChild<T: Bundle + Default> {
    child: Option<T>,
}

impl<T: Bundle + Default> WithChild<T> {
    pub fn new(value: T) -> Self {
        Self { child: Some(value) }
    }
}

impl<T: Bundle + Default> Default for WithChild<T> {
    fn default() -> Self {
        Self {
            child: Some(default()),
        }
    }
}

impl<T: Bundle + Default> Component for WithChild<T> {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_add(|mut world, entity, _component_id| {
            let bundle = world.get_mut::<WithChild<T>>(entity).unwrap().child.take();

            let mut w = world.commands();
            let mut commands = w.entity(entity);

            if let Some(bundle) = bundle {
                commands.with_children(|parent| {
                    parent.spawn(bundle);
                });
            }

            commands.remove::<WithChild<T>>();
        });
    }
}
