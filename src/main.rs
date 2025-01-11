use std::env;

use bevy::{prelude::*, window::WindowMode};
mod camera_controller;
mod control;
mod entities;
mod model;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use camera_controller::{CameraControllerPlugin, Spectate};
use control::{Control, ControlPlugin, Input};
use entities::{
    block::BlockBundle,
    ghost::{GhostBundle, GhostPlugin},
};
use levels::test_level::TestLevelBundle;
use linker::LinkerPlugin;
use liquid::{Buoyant, Liquid, LiquidPlugin};
use model::ModelPlugin;
use ray_caster::RayCasterPlugin;
use shape_caster::ShapeCasterPlugin;
use throttle::ThrottlePlugin;
mod levels;
mod library;
mod linker;
mod liquid;
mod ray_caster;
mod shape_caster;
mod throttle;
mod with_child;
mod with_material;
mod with_mesh;

#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub enum AppSystems {
    Startup,
    Update,
}

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct Debugging {
    enable: bool,
}

fn main() {
    let mut app = App::new();
    let mut app = app
        .add_plugins((DefaultPlugins, RapierPhysicsPlugin::<NoUserData>::default()))
        .add_plugins((
            ModelPlugin,
            GhostPlugin,
            CameraControllerPlugin,
            ControlPlugin,
            ShapeCasterPlugin,
            LinkerPlugin,
            ThrottlePlugin,
            RayCasterPlugin,
            LiquidPlugin,
        ))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 100.0,
        })
        .insert_resource(Debugging::default())
        .insert_resource(ClearColor(Color::srgb(0.8, 0.9, 1.0)))
        .add_systems(Startup, startup.in_set(AppSystems::Startup))
        .add_systems(PreUpdate, screen_mode_update.in_set(AppSystems::Update));

    let debug = env::var("DEBUG").unwrap_or("".to_owned());

    if debug == "true" {
        app = app
            .add_plugins((
                RapierDebugRenderPlugin::default(),
                WorldInspectorPlugin::default(),
            ))
            .register_type::<Debugging>()
            .insert_resource(Debugging { enable: true });
    }

    app.run();
}

fn screen_mode_update(mut input: ResMut<Input>, mut window: Single<&mut Window>) {
    if let WindowMode::BorderlessFullscreen(_) = window.mode {
        let x = window.resolution.width() / 2.0;
        let y = window.resolution.height() / 2.0;
        window.set_cursor_position(Some(Vec2::new(x, y)));
    }

    if !input.full_screen_switching() {
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

fn startup(mut commands: Commands) {
    commands.spawn(TestLevelBundle::default());
    commands.spawn((
        GhostBundle::new(),
        Transform::from_xyz(0.0, 3.0, 0.0),
        Buoyant::new(1.0),
        Spectate,
        Control,
    ));

    commands.spawn((
        Collider::cuboid(10.0, 1.0, 10.0),
        ActiveEvents::COLLISION_EVENTS,
        Liquid::new(0.01),
        SolverGroups::new(Group::NONE, Group::NONE),
        Transform::from_xyz(0.0, 0.0, 20.0),
    ));

    commands.spawn((
        BlockBundle::default(),
        Transform::from_xyz(0.0, 2.0, 20.0),
        Buoyant::new(0.4),
        Velocity::default(),
    ));

    commands.spawn((
        BlockBundle::new(3.0, 0.5, 3.0),
        Transform::from_xyz(3.0, 2.0, 20.0),
        Buoyant::new(0.4),
        Velocity::default(),
    ));

    commands.spawn((
        BlockBundle::new(3.0, 0.5, 3.0),
        Transform::from_xyz(3.0, 3.0, 20.0),
        Buoyant::new(0.4),
        Velocity::default(),
    ));
}
