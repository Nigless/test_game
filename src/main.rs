use std::f32::consts;

use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    window::WindowMode,
};
mod camera_controller;
mod control;
mod entities;
mod model;
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
            RapierPhysicsPlugin::<NoUserData>::default(),
            UiMaterialPlugin::<CrosshairMaterial>::default(),
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
            color: Color::rgb(1.0, 1.0, 1.0),
            brightness: 100.0,
        })
        .insert_resource(ClearColor(Color::rgb(0.8, 0.9, 1.0)))
        .add_systems(PreStartup, startup)
        .add_systems(PreUpdate, screen_mode_update)
        .run();
}

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
struct CrosshairMaterial {}

impl UiMaterial for CrosshairMaterial {
    fn fragment_shader() -> ShaderRef {
        "crosshair.wgsl".into()
    }
}

fn screen_mode_update(input: Res<Input>, mut window_q: Query<&mut Window>) {
    let mut window = window_q.get_single_mut().unwrap();

    if window.mode == WindowMode::Fullscreen {
        let x = window.resolution.width() / 2.0;
        let y = window.resolution.height() / 2.0;
        window.set_cursor_position(Some(Vec2::new(x, y)));
    }

    if !input.full_screen_switching {
        return;
    }

    if window.mode == WindowMode::Fullscreen {
        window.cursor.visible = true;
        window.mode = WindowMode::Windowed;

        return;
    }

    window.cursor.visible = false;
    window.mode = WindowMode::Fullscreen
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
            let size = 1.0;

            commands.spawn(MaterialNodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    width: Val::VMin(size),
                    height: Val::VMin(size),
                    top: Val::VMin(size / -2.0),
                    left: Val::VMin(size / -2.0),
                    ..default()
                },
                material: crosshair_materials.add(CrosshairMaterial {}),
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
            illuminance: 3000.0,
            shadows_enabled: true,
            color: Color::rgb(1.0, 1.0, 1.0),
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
        .spawn(GhostBundle::new())
        .insert(Transform::from_xyz(0.0, 3.0, 0.0))
        .insert(Spectate)
        .insert(Control);

    commands.spawn(GhostBundle::new()).insert(
        Transform::from_xyz(4.0, 2.2, 5.0).with_rotation(Quat::from_rotation_y(consts::PI)),
    );
}
