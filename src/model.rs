use bevy::prelude::*;
use bevy_rapier3d::geometry::{Collider, ComputedColliderShape};

#[derive(Component)]
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

#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub enum ModelSystems {
    Resolve,
}

pub struct ModelPlugin;

impl Plugin for ModelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(First, (load, resolve).in_set(ModelSystems::Resolve));
    }
}

fn load(server: Res<AssetServer>, mut commands: Commands, models_q: Query<(Entity, &Model)>) {
    for (entity, model) in models_q.iter() {
        let src = model.src.clone();

        commands.entity(entity).remove::<Model>().insert(SceneRoot(
            server.load(GltfAssetLabel::Scene(0).from_asset(src)),
        ));
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

        if extras.value != "{\"Collider\":true}" {
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
