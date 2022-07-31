use crate::camera::CameraPlugin;
use crate::control::ControlPlugin;
use crate::physics::PhysicsPlugin;
use bevy::prelude::*;

mod camera;
mod components;
mod control;
mod entities;
mod physics;
use control::Control;
use entities::player::Player;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ControlPlugin)
        .add_plugin(CameraPlugin)
        .add_plugin(PhysicsPlugin)
        .add_startup_system(setup)
        .insert_resource(AmbientLight {
            color: Color::rgb(1.0, 1.0, 1.0),
            brightness: 0.9,
        })
        .insert_resource(ClearColor(Color::rgb(0.8, 0.8, 0.8)))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn_bundle(Player::new("sanek".to_owned()))
        .insert(Control);
    for x in -5..5 {
        for z in -5..5 {
            commands.spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Plane { size: 1.0 })),

                material: materials.add(StandardMaterial {
                    base_color: Color::GREEN,
                    reflectance: 0.0,
                    unlit: false,
                    ..Default::default()
                }),
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
