use bevy::{
    ecs::component::{ComponentHooks, StorageType},
    prelude::*,
};
use bevy_rapier3d::geometry::{Collider, ComputedColliderShape};

pub struct Model {
    pub src: String,
}

impl Model {
    pub fn new<'a>(src: &'a str) -> Self {
        Self {
            src: src.to_owned(),
        }
    }
}

impl Component for Model {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_add(|mut world, entity, _component_id| {
            let src = world.get_mut::<Model>(entity).unwrap().src.clone();

            world.commands().entity(entity).remove::<Model>();

            let server = world.get_resource::<AssetServer>().unwrap();

            let scene_root = SceneRoot(server.load(GltfAssetLabel::Scene(0).from_asset(src)));

            world
                .commands()
                .entity(entity)
                .remove::<Model>()
                .insert(scene_root);
        });
    }
}

#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub enum ModelSystems {
    Resolve,
}

pub struct ModelPlugin;

impl Plugin for ModelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(First, resolve.in_set(ModelSystems::Resolve));
    }
}

fn resolve(
    meshes_res: Res<Assets<Mesh>>,
    mut commands: Commands,
    models_q: Query<(Entity, &Children, &GltfExtras)>,
    meshes_q: Query<&Mesh3d>,
) {
    for (entity, children, extras) in models_q.iter() {
        commands.entity(entity).remove::<GltfExtras>();

        if extras.value != "{\"collider\":true}" {
            continue;
        }

        let mesh = meshes_q.get(children[0]).unwrap();

        commands.entity(children[0]).despawn();

        commands
            .entity(entity)
            .insert(
                Collider::from_bevy_mesh(
                    meshes_res.get(mesh).unwrap(),
                    &ComputedColliderShape::default(),
                )
                .unwrap(),
            )
            .remove::<Children>();
    }
}
