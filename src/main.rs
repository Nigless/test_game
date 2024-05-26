use std::f32::consts;

use bevy::{
    prelude::*,
    render::{
        camera::CameraRenderGraph,
        render_resource::{AsBindGroup, ShaderRef},
        view::VisibleEntities,
    },
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
mod camera_controller;
mod control;
mod entities;
mod model;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use camera_controller::{CameraControllerPlugin, Spectate};
use character_body::{CharacterBody, CharacterBodyPlugin};
use control::{Bindings, Control, ControlPlugin, Input};
use entities::{
    ghost::{Ghost, GhostPlugin},
    traffic_cone::TrafficCone,
};
use model::{Model, ModelPlugin};
mod character_body;

use crate::entities::package::Package;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            WorldInspectorPlugin::new(),
            RapierPhysicsPlugin::<NoUserData>::default(),
            // RapierDebugRenderPlugin::default(),
            UiMaterialPlugin::<CrosshairMaterial>::default(),
        ))
        .add_plugins((
            ModelPlugin,
            GhostPlugin,
            CameraControllerPlugin,
            ControlPlugin,
            CharacterBodyPlugin,
        ))
        .insert_resource(AmbientLight {
            color: Color::rgb(1.0, 1.0, 1.0),
            brightness: 500.0,
        })
        .insert_resource(ClearColor(Color::rgb(0.7, 0.7, 0.7)))
        .add_systems(PreStartup, startup)
        .run();
}

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
struct CrosshairMaterial {
    #[uniform(0)]
    color: Vec4,
}

impl UiMaterial for CrosshairMaterial {
    fn fragment_shader() -> ShaderRef {
        "crosshair.wgsl".into()
    }
}

fn startup(mut commands: Commands, mut crosshair_materials: ResMut<Assets<CrosshairMaterial>>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                top: Val::Percent(50.0),
                left: Val::Percent(50.0),
                ..default()
            },
            ..default()
        })
        .with_children(|commands| {
            commands.spawn(MaterialNodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    width: Val::VMin(0.7),
                    height: Val::VMin(0.7),
                    top: Val::VMin(-0.35),
                    left: Val::VMin(-0.35),
                    ..default()
                },
                material: crosshair_materials.add(CrosshairMaterial {
                    color: Color::DARK_GRAY.rgba_to_vec4(),
                }),
                ..default()
            });
            commands.spawn(MaterialNodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    width: Val::VMin(0.4),
                    height: Val::VMin(0.4),
                    top: Val::VMin(-0.2),
                    left: Val::VMin(-0.2),
                    ..default()
                },
                material: crosshair_materials.add(CrosshairMaterial {
                    color: Color::WHITE.rgba_to_vec4(),
                }),
                ..default()
            });
        });

    commands.spawn((
        Model::new("test_scene.glb"),
        RigidBody::Fixed,
        TransformBundle::default(),
    ));

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 2000.0,
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
