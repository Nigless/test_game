use std::{env, time::Duration};

use bevy::{
    color::palettes::css::RED,
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    pbr::NotShadowCaster,
    prelude::*,
    render::primitives::Aabb,
    window::WindowMode,
};
mod billboard;
mod camera_controller;
mod control;
mod entities;
mod model;
use bevy_hanabi::HanabiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use billboard::BillboardPlugin;
use camera_controller::{CameraControllerPlugin, Spectate};
use control::{Control, ControlPlugin, Input};
use despawn::{Despawn, DespawnPlugin};
use entities::{
    block::BlockBundle,
    fireball::{Fireball, FireballPlugin},
    player::{Player, PlayerPlugin},
};
use explosion::ExplosionPlugin;
use levels::test_level::TestLevelBundle;
use library::Spawnable;
use linker::LinkerPlugin;
use model::ModelPlugin;
use random::RandomPlugin;
use ray_caster::RayCasterPlugin;
use shape_caster::ShapeCasterPlugin;
use throttle::ThrottlePlugin;
use tracy_client::Client;
use with_material::WithMaterial;
use with_mesh::WithMesh;
mod despawn;
mod explosion;
mod levels;
mod library;
mod linker;
mod random;
mod ray_caster;
mod shape_caster;
mod throttle;
mod with_material;
mod with_mesh;

#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub enum AppSystems {
    Startup,
    Update,
}

fn main() {
    let _client = Client::start();

    let mut app = App::new();
    let mut app = app
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            RapierPhysicsPlugin::<NoUserData>::default(),
            HanabiPlugin,
        ))
        .add_plugins((
            ModelPlugin,
            PlayerPlugin,
            FireballPlugin,
            CameraControllerPlugin,
            ControlPlugin,
            ShapeCasterPlugin,
            LinkerPlugin,
            ThrottlePlugin,
            RayCasterPlugin,
            RandomPlugin::default(),
            DespawnPlugin,
            BillboardPlugin,
            ExplosionPlugin,
        ))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 100.0,
        })
        .insert_resource(ClearColor(Color::srgb(0.8, 0.9, 1.0)))
        .add_systems(Startup, startup.in_set(AppSystems::Startup))
        .add_systems(PreUpdate, screen_mode_update.in_set(AppSystems::Update));

    #[cfg(debug_assertions)]
    {
        app = app.add_plugins((
            RapierDebugRenderPlugin::default(),
            WorldInspectorPlugin::default(),
            FpsOverlayPlugin {
                config: FpsOverlayConfig {
                    text_color: RED.into(),
                    text_config: TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    ..default()
                },
            },
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
    TestLevelBundle.spawn(&mut commands);
}
