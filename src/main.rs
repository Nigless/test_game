use crate::camera::CameraPlugin;
use crate::control::ControlPlugin;
use bevy::prelude::*;
use bevy::render::mesh::skinning::SkinnedMesh;
use bevy::render::settings::Backends;
use bevy::render::settings::WgpuSettings;
use bevy::scene::*;
use bevy_editor_pls::prelude::*;
use bevy_rapier3d::prelude::RigidBody;
mod camera;
mod components;
mod control;
mod entities;
mod head;
mod model;
mod sprite;
mod utils;
use bevy_rapier3d::prelude::*;
use camera::CameraTarget;
use control::Control;
use entities::bullet::Bullet;
use entities::player::Player;
use head::HeadPlugin;
use model::ModelPlugin;
use sprite::SpritePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(AmbientLight {
            color: Color::rgb(1.0, 1.0, 1.0),
            brightness: 0.9,
        })
        .insert_resource(ClearColor(Color::rgb(0.8, 0.8, 0.8)))
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_startup_system(startup)
        .add_plugin(EditorPlugin)
        .add_plugin(CameraPlugin)
        .add_plugin(ControlPlugin)
        .add_plugin(HeadPlugin)
        .add_plugin(ModelPlugin)
        .add_plugin(SpritePlugin)
        .run();
}

fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn_bundle(Player::new())
        .insert(Control)
        .insert(CameraTarget)
        .insert(Visibility { is_visible: false });

    commands.spawn().insert(Collider::cuboid(500.0, 0.1, 500.0));

    commands.spawn_bundle(Bullet::new());
    // commands
    //     .spawn()
    //     .insert(Velocity::default())
    //     .insert(GlobalTransform::default())
    //     .insert(Transform::from_xyz(0.0, 1.0, 0.0))
    //     .insert(Control)
    //     .insert(CameraTarget)
    //     .insert(RigidBody::Dynamic)
    //     .insert(Collider::cuboid(0.3, 0.3, 0.3));

    let mesh = meshes.add(Mesh::from(shape::Plane { size: 1.0 }));
    let material = materials.add(StandardMaterial {
        base_color: Color::BLACK,
        reflectance: 0.0,
        unlit: false,
        ..Default::default()
    });
    for x in -5..5 {
        for z in -5..5 {
            commands.spawn_bundle(PbrBundle {
                mesh: mesh.clone(),
                material: material.clone(),
                transform: Transform::from_translation(Vec3::new(
                    x as f32 * 5.0,
                    0.0,
                    z as f32 * 5.0,
                )),
                ..Default::default()
            });
        }
    }
}
