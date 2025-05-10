use bevy::{
    ecs::component::{ComponentHooks, StorageType},
    prelude::*,
};

#[derive(Clone)]
pub struct WithMaterial {
    material: Option<StandardMaterial>,
}

impl WithMaterial {
    pub fn new(material: impl Into<StandardMaterial>) -> Self {
        Self {
            material: Some(material.into()),
        }
    }
}

impl Component for WithMaterial {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_add(|mut world, entity, _| {
            let material = world
                .get_mut::<WithMaterial>(entity)
                .unwrap()
                .material
                .take();

            let mut materials = world
                .get_resource_mut::<Assets<StandardMaterial>>()
                .unwrap();

            if let Some(material) = material {
                let handle = materials.add(material);

                world
                    .commands()
                    .entity(entity)
                    .insert(MeshMaterial3d(handle));
            }

            world.commands().entity(entity).remove::<WithMaterial>();
        });
    }
}
