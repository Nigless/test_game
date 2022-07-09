mod components;
use bevy::{app::App, prelude::Commands, window::WindowDescriptor, DefaultPlugins};
mod control;
use bevy::prelude::*;
mod entities;
use control::Control;
use control::ControlPlugin;
use entities::player::Player;
mod camera;
mod physics;
use camera::CameraPlugin;
use physics::PhysicsPlugin;
use world::world::WorldPlugin;
mod world;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ControlPlugin())
        .add_plugin(CameraPlugin())
        .add_plugin(PhysicsPlugin())
        .add_plugin(WorldPlugin())
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
        .insert(Control());

    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 100.0 })),
        material: materials.add(StandardMaterial {
            base_color: Color::GREEN,
            reflectance: 0.02,
            unlit: false,
            ..Default::default()
        }),
        transform: Transform::from_translation(Vec3::new(10.0, 0.0, 0.0)),
        ..Default::default()
    });
    // commands.spawn_bundle(PointLightBundle {
    //     transform: Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
    //     ..Default::default()
    // });

    // commands.spawn_bundle(PerspectiveCameraBundle {
    //     transform: Transform::from_translation(Vec3::new(0.0, 0.0, 15.0))
    //         .looking_at(Vec3::default(), Vec3::Y),
    //     ..Default::default()
    // });
}
