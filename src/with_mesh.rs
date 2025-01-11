use bevy::{
    ecs::component::{ComponentHooks, StorageType},
    prelude::*,
};

pub struct WithMesh {
    mesh: Option<Mesh>,
}

impl WithMesh {
    pub fn new(mesh: impl Into<Mesh>) -> Self {
        Self {
            mesh: Some(mesh.into()),
        }
    }
}

impl Component for WithMesh {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_add(|mut world, entity, _component_id| {
            let mesh = world.get_mut::<WithMesh>(entity).unwrap().mesh.take();

            let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();

            if let Some(mesh) = mesh {
                let handle = meshes.add(mesh);

                world.commands().entity(entity).insert(Mesh3d(handle));
            }

            world.commands().entity(entity).remove::<WithMesh>();
        });
    }
}
