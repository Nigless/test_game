use bevy::prelude::*;
mod components;
mod control;
mod entities;
mod model;
mod utils;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use control::Controls;
use entities::ghost::{Ghost, GhostPlugin};
use model::ModelPlugin;

use crate::entities::package::Package;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            WorldInspectorPlugin::new(),
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default(),
        ))
        .add_plugins((ModelPlugin, GhostPlugin))
        .add_systems(Startup, startup)
        .insert_resource(AmbientLight {
            color: Color::rgb(1.0, 1.0, 1.0),
            brightness: 0.9,
        })
        .insert_resource(ClearColor(Color::rgb(0.8, 0.8, 0.8)))
        .insert_resource(Controls::default())
        .run();
}

fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let velocity = Vec3::new(-2.3, -5.12, 0.0);
    let normal = Vec3::new(0.3, 1.0, 0.0).normalize();

    println!("{}", velocity - velocity.project_onto(normal));
    // > [-0, 0, 1.9940411]

    commands
        .spawn(Ghost::new())
        .insert(Transform::from_xyz(0.0, 10.0, 0.0));

    commands
        .spawn(Package::new())
        .insert(Transform::from_xyz(3.0, 2.0, 0.0));

    commands.spawn(Collider::cuboid(500.0, 0.1, 500.0));

    let mesh = meshes.add(Mesh::from(Plane3d {
        normal: Direction3d::try_from(Vec3::new(0.0, 1.0, 0.0)).unwrap(),
    }));

    let material = materials.add(StandardMaterial {
        base_color: Color::BLACK,
        reflectance: 0.0,
        unlit: false,
        ..Default::default()
    });

    for x in -5..5 {
        for z in -5..5 {
            commands.spawn(PbrBundle {
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
