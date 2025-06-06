use bevy::{
    ecs::{component::ComponentId, world::DeferredWorld},
    prelude::*,
};
use bevy_rapier3d::prelude::*;

#[derive(Resource, Clone)]
struct BlockAssets {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
#[component(on_add = resolve)]
pub struct Block;

pub struct BlockPlugin;

impl Plugin for BlockPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, startup);
    }
}

fn startup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.insert_resource(BlockAssets {
        material: materials.add(Color::srgb_u8(255, 255, 255)),
        mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
    });
}

fn resolve(mut world: DeferredWorld<'_>, entity: Entity, _: ComponentId) {
    let assets = world.get_resource::<BlockAssets>().cloned().unwrap();

    world.commands().entity(entity).insert_if_new((
        Name::new("block"),
        Collider::cuboid(0.5, 0.5, 0.5),
        RigidBody::Dynamic,
        ColliderMassProperties::Mass(5.0),
        Velocity::default(),
        Transform::default(),
        Mesh3d(assets.mesh),
        MeshMaterial3d(assets.material),
    ));
}
