use std::f32::consts;

use bevy::prelude::*;
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
use model::{Model, ModelPlugin};

use crate::entities::package::Package;

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
            brightness: 100.0,
        })
        .insert_resource(ClearColor(Color::rgb(1.0, 1.0, 1.0)))
        .insert_resource(Bindings::default())
        .add_systems(PreStartup, startup)
        .run();
}

fn startup(mut commands: Commands) {
    commands.spawn((
        Model::new("test_scene.glb"),
        RigidBody::Fixed,
        TransformBundle::default(),
    ));

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            rotation: Quat::from_rotation_y(consts::PI * -0.1)
                * Quat::from_rotation_x(consts::PI * -0.6),
            ..default()
        },
        ..default()
    });

    commands
        .spawn(Ghost::new())
        .insert(Transform::from_xyz(0.0, 3.0, 0.0))
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
}
