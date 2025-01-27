use std::env;

use bevy::{pbr::NotShadowCaster, prelude::*, render::primitives::Aabb, window::WindowMode};
mod camera_controller;
mod control;
mod entities;
mod model;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use camera_controller::{CameraControllerPlugin, Spectate};
use control::{Control, ControlPlugin, Input};
use despawn::{Despawn, DespawnPlugin};
use entities::{
    block::BlockBundle,
    player::{GhostPlugin, Player},
};
use levels::test_level::TestLevelBundle;
use library::Spawnable;
use linker::LinkerPlugin;
use liquid::{Liquid, LiquidPlugin};
use model::ModelPlugin;
use random::RandomPlugin;
use ray_caster::RayCasterPlugin;
use shape_caster::ShapeCasterPlugin;
use throttle::ThrottlePlugin;
use with_material::WithMaterial;
use with_mesh::WithMesh;
mod despawn;
mod levels;
mod library;
mod linker;
mod liquid;
mod random;
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
            RandomPlugin::default(),
            DespawnPlugin,
        ))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 100.0,
        })
        .insert_resource(ClearColor(Color::srgb(0.8, 0.9, 1.0)))
        .add_systems(PreStartup, startup.in_set(AppSystems::Startup))
        .add_systems(PreUpdate, screen_mode_update.in_set(AppSystems::Update));

    #[cfg(debug_assertions)]
    {
        app = app.add_plugins((
            RapierDebugRenderPlugin::default(),
            WorldInspectorPlugin::default(),
        ));
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
    commands.spawn(TestLevelBundle::bundle());

    let player = Player.spawn(&mut commands);

    commands
        .entity(player)
        .insert((Transform::from_xyz(0.0, 3.0, 0.0), Spectate, Control));

    commands.spawn((
        Collider::cuboid(10.0, 1.0, 10.0),
        WithMesh::new(Cuboid::new(20.0, 2.0, 20.0)),
        WithMaterial::new(Color::srgba(0.7, 0.7, 1.0, 0.2)),
        ActiveEvents::COLLISION_EVENTS,
        NotShadowCaster,
        Sensor,
        Liquid::new(1000.0).with_sample_count(200),
        Transform::from_xyz(0.0, 0.0, 20.0),
    ));

    commands.spawn((
        BlockBundle::default(),
        Transform::from_xyz(-4.0, 3.0, 16.0),
        Velocity::default(),
    ));

    commands.spawn((
        BlockBundle::default(),
        Transform::from_xyz(-4.0, 3.0, 24.0),
        Velocity::default(),
    ));

    commands.spawn((
        BlockBundle::default(),
        Transform::from_xyz(4.0, 3.0, 16.0),
        Velocity::default(),
    ));

    commands.spawn((
        BlockBundle::new(3.0, 0.2, 3.0),
        Transform::from_xyz(4.0, 3.0, 24.0),
        Velocity::default(),
    ));

    commands.spawn((
        BlockBundle::new(0.5, 0.5, 0.5).with_mass(25.0 / 2.0),
        Transform::from_xyz(0.0, 2.0, 20.0),
        Velocity::default(),
    ));
}
