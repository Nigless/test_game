use bevy::{gltf::GltfMesh, prelude::*};
use bevy_rapier3d::geometry::{AsyncSceneCollider, Collider, ComputedColliderShape};

#[derive(Component)]
pub struct HitBox {
    pub src: String,
}

impl HitBox {
    pub fn new<'a>(src: &'a str) -> Self {
        Self {
            src: src.to_owned(),
        }
    }
}

pub struct HitBoxPlugin;

impl Plugin for HitBoxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(First, resolve);
    }
}

fn resolve(
    server: Res<AssetServer>,
    meshes: Res<Assets<Mesh>>,
    mut commands: Commands,
    models_q: Query<(Entity, &HitBox)>,
) {
    for (entity, hit_box) in models_q.iter() {
        let mesh = match meshes.get(server.load::<Mesh>(hit_box.src.clone())) {
            Some(mesh) => mesh,
            None => continue,
        };

        let collider = match Collider::from_bevy_mesh(mesh, &ComputedColliderShape::default()) {
            Some(collider) => collider,
            None => continue,
        };

        commands.entity(entity).remove::<HitBox>().insert(collider);
    }
}
