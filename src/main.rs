use std::f32::consts;

use bevy::{
    prelude::*,
    render::{mesh::PlaneMeshBuilder, render_resource::ShaderType},
};
mod bindings;
mod camera_controller;
mod components;
mod entities;
mod model;
mod utils;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use bindings::{Bindings, Control};
use camera_controller::{CameraControllerPlugin, Spectate};
use entities::{
    ghost::{Ghost, GhostPlugin},
    traffic_cone::TrafficCone,
};
use model::ModelPlugin;

use crate::{camera_controller::CameraController, entities::package::Package};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            WorldInspectorPlugin::new(),
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default(),
        ))
        .add_plugins((ModelPlugin, GhostPlugin, CameraControllerPlugin))
        .insert_resource(AmbientLight {
            color: Color::rgb(1.0, 1.0, 1.0),
            brightness: 1000.0,
        })
        .insert_resource(ClearColor(Color::rgb(1.0, 1.0, 1.0)))
        .insert_resource(Bindings::default())
        .add_systems(PreStartup, startup)
        .run();
}

fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn(Ghost::new())
        .insert(Transform::from_xyz(0.0, 10.0, 0.0))
        .insert(Spectate)
        .insert(Control);

    commands.spawn(Ghost::new()).insert(
        Transform::from_xyz(10.0, 10.0, 0.0).with_rotation(Quat::from_rotation_y(consts::PI)),
    );

    commands
        .spawn(Package::new())
        .insert(Transform::from_xyz(3.0, 2.0, 0.0));

    commands
        .spawn(TrafficCone::new())
        .insert(Transform::from_xyz(3.0, 2.0, 5.0));
    commands
        .spawn(TrafficCone::new())
        .insert(Transform::from_xyz(3.0, 3.0, 5.0));
    commands
        .spawn(TrafficCone::new())
        .insert(Transform::from_xyz(3.0, 4.0, 5.0));

    commands.spawn(Collider::cuboid(500.0, 0.1, 500.0));

    let material = materials.add(StandardMaterial {
        base_color: Color::RED,
        ..Default::default()
    });

    let mesh = meshes.add(Mesh::from(PlaneMeshBuilder::from_size(Vec2::new(1.0, 1.0))));

    for x in -10..10 {
        for z in -10..10 {
            commands.spawn(PbrBundle {
                mesh: mesh.clone(),
                material: material.clone(),
                transform: Transform::from_translation(Vec3::new(
                    x as f32 * 2.0,
                    0.0,
                    z as f32 * 2.0,
                )),
                ..Default::default()
            });
        }
    }
}
