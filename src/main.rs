use std::f32::consts;

use bevy::{prelude::*, window::WindowMode};
mod camera_controller;
mod control;
mod entities;
mod model;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use camera_controller::{CameraControllerPlugin, Spectate};
use character_body::CharacterBodyPlugin;
use control::{Control, ControlPlugin, Input};
use entities::ghost::{GhostBundle, GhostPlugin};
use linker::LinkerPlugin;
use model::{Model, ModelPlugin};
use shape_caster::ShapeCasterPlugin;
mod character_body;
mod lib;
mod linker;
mod shape_caster;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            WorldInspectorPlugin::default(),
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default(),
        ))
        .add_plugins((
            ModelPlugin,
            GhostPlugin,
            CameraControllerPlugin,
            ControlPlugin,
            CharacterBodyPlugin,
            ShapeCasterPlugin,
            LinkerPlugin,
        ))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 100.0,
        })
        .insert_resource(ClearColor(Color::srgb(0.8, 0.9, 1.0)))
        .add_systems(PreStartup, startup)
        .add_systems(PreUpdate, screen_mode_update)
        .run();
}

fn screen_mode_update(input: Res<Input>, mut window: Single<&mut Window>) {
    if let WindowMode::BorderlessFullscreen(_) = window.mode {
        let x = window.resolution.width() / 2.0;
        let y = window.resolution.height() / 2.0;
        window.set_cursor_position(Some(Vec2::new(x, y)));
    }

    if !input.full_screen_switching {
        return;
    }

    if let WindowMode::BorderlessFullscreen(_) = window.mode {
        window.cursor_options.visible = true;
        window.mode = WindowMode::Windowed;

        return;
    }

    window.cursor_options.visible = false;
    window.mode = WindowMode::BorderlessFullscreen(MonitorSelection::Current)
}

fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((Model::new("test_scene.glb"), RigidBody::Fixed));

    commands.spawn((
        DirectionalLight {
            illuminance: 3000.0,
            shadows_enabled: true,
            color: Color::WHITE,
            ..default()
        },
        Transform::from_rotation(
            Quat::from_rotation_y(consts::PI * -0.1) * Quat::from_rotation_x(consts::PI * -0.6),
        ),
    ));

    commands
        .spawn(GhostBundle::new())
        .insert(Transform::from_xyz(0.0, 3.0, 0.0))
        .insert(Spectate)
        .insert(Control);

    commands.spawn(GhostBundle::new()).insert(
        Transform::from_xyz(4.0, 2.2, 5.0).with_rotation(Quat::from_rotation_y(consts::PI)),
    );

    commands.spawn((
        Name::new("Package"),
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Collider::cuboid(0.5, 0.5, 0.5),
        RigidBody::Dynamic,
        Transform::from_xyz(0.0, 10.0, 10.0),
        Velocity::default(),
    ));
}
