mod components;
use bevy::{app::App, prelude::Commands, window::WindowDescriptor, DefaultPlugins};
mod control;
use bevy::prelude::*;
mod entitys;
use control::Control;
use control::ControlPlugin;
use entitys::player::Player;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ControlPlugin())
        .add_startup_system(setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube_size = 4.0;

    commands
        .spawn_bundle(Player::new("sanek".to_owned()))
        .insert(Control());

    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box::new(cube_size, cube_size, cube_size))),
        material: materials.add(StandardMaterial {
            base_color: Color::BLUE,
            reflectance: 0.02,
            unlit: false,
            ..Default::default()
        }),
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, -10.0)),
        ..Default::default()
    });
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box::new(cube_size, cube_size, cube_size))),
        material: materials.add(StandardMaterial {
            base_color: Color::BLUE,
            reflectance: 0.02,
            unlit: false,
            ..Default::default()
        }),
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
        ..Default::default()
    });
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box::new(cube_size, cube_size, cube_size))),
        material: materials.add(StandardMaterial {
            base_color: Color::GREEN,
            reflectance: 0.02,
            unlit: false,
            ..Default::default()
        }),
        transform: Transform::from_translation(Vec3::new(0.0, -10.0, 0.0)),
        ..Default::default()
    });
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box::new(cube_size, cube_size, cube_size))),
        material: materials.add(StandardMaterial {
            base_color: Color::GREEN,
            reflectance: 0.02,
            unlit: false,
            ..Default::default()
        }),
        transform: Transform::from_translation(Vec3::new(0.0, 10.0, 0.0)),
        ..Default::default()
    });
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box::new(cube_size, cube_size, cube_size))),
        material: materials.add(StandardMaterial {
            base_color: Color::RED,
            reflectance: 0.02,
            unlit: false,
            ..Default::default()
        }),
        transform: Transform::from_translation(Vec3::new(-10.0, 0.0, 0.0)),
        ..Default::default()
    });
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box::new(cube_size, cube_size, cube_size))),
        material: materials.add(StandardMaterial {
            base_color: Color::RED,
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
