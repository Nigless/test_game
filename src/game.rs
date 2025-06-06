use bevy::{
    color::palettes::{basic, css::RED},
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    prelude::*,
    window::WindowMode,
};
use bevy_hanabi::{EffectSimulation, EffectSimulationTime, HanabiPlugin};
use bevy_rapier3d::{
    plugin::{NoUserData, RapierConfiguration, RapierPhysicsPlugin},
    prelude::Velocity,
    render::{DebugRenderContext, RapierDebugRenderPlugin},
};

use crate::{
    entities::{
        block::Block, fireball::Fireball, gas_can::GasCan, player::Player,
        traffic_cone::TrafficCone, EntitiesPlugin,
    },
    plugins::{
        input::{Control, FullScreenSwitchingInvokedEvent, Input},
        prefab::{Prefab, PrefabsLoadedEvent},
        serializable::Serializable,
        settings::Settings,
    },
    scenes::ScenesPlugin,
    stores::{pause::PauseState, StoresPlugin},
};

use super::plugins::*;

pub fn game() -> App {
    let mut _app = App::new();
    let mut app = _app
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            RapierPhysicsPlugin::<NoUserData>::default(),
            HanabiPlugin,
            ThrottlePlugin,
            RapierDebugRenderPlugin::default(),
            FpsOverlayPlugin {
                config: FpsOverlayConfig {
                    text_color: basic::RED.into(),
                    text_config: TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    ..default()
                },
            },
        ))
        .add_plugins((
            InputPlugin,
            ShapeCasterPlugin,
            LinkerPlugin,
            RayCasterPlugin,
            RandomPlugin::default(),
            DespawnPlugin,
            BillboardPlugin,
            EntitiesPlugin,
            HealthPlugin,
            SceneControllerPlugin,
            CollisionEventsPlugins,
            StoresPlugin,
            ScenesPlugin,
            SettingsPlugin,
        ))
        .add_plugins((SerializablePlugin, PrefabPlugin))
        .add_systems(
            First,
            (
                switch_pause.run_if(resource_changed::<PauseState>),
                switch_colliders_debug.run_if(resource_changed::<Settings>),
            ),
        )
        .add_observer(handle_loaded)
        .add_observer(handle_full_screen_switch);

    #[cfg(debug_assertions)]
    {
        use bevy_inspector_egui::quick::WorldInspectorPlugin;

        app = app.add_plugins(WorldInspectorPlugin::default());
    }

    _app
}

fn switch_colliders_debug(settings: Res<Settings>, mut rapier_debug: ResMut<DebugRenderContext>) {
    rapier_debug.enabled = settings.dev_settings.show_colliders;
}

fn handle_full_screen_switch(
    _: Trigger<FullScreenSwitchingInvokedEvent>,
    mut window: Single<&mut Window>,
) {
    if let WindowMode::BorderlessFullscreen(_) = window.mode {
        let x = window.resolution.width() / 2.0;
        let y = window.resolution.height() / 2.0;
        window.set_cursor_position(Some(Vec2::new(x, y)));
    }

    if let WindowMode::BorderlessFullscreen(_) = window.mode {
        window.cursor_options.visible = true;
        window.mode = WindowMode::Windowed;

        return;
    }

    window.cursor_options.visible = false;
    window.mode = WindowMode::BorderlessFullscreen(MonitorSelection::Current)
}

fn switch_pause(
    mut rapier_config: Single<&mut RapierConfiguration>,
    pause_game: Res<PauseState>,
    mut effects_time: ResMut<Time<EffectSimulation>>,
) {
    match *pause_game {
        PauseState::Pause => {
            rapier_config.physics_pipeline_active = false;
            effects_time.pause();
        }
        PauseState::Playing => {
            rapier_config.physics_pipeline_active = true;
            effects_time.unpause();
        }
    }
}

fn handle_loaded(_: Trigger<PrefabsLoadedEvent>, mut commands: Commands) {
    let commands = &mut commands;

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 100.0,
    });
    commands.insert_resource(ClearColor(Color::srgb(0.8, 0.9, 1.0)));

    commands.spawn((
        Serializable::default().with::<Prefab>(),
        Prefab::new("test_scene"),
    ));

    commands
        .spawn(Player::default())
        .insert((Transform::from_xyz(0.0, 3.0, 0.0), Control));

    commands.spawn(Fireball).insert((
        Transform::from_xyz(0.0, 1.0, 3.0),
        Velocity::linear(Vec3::X),
    ));

    commands
        .spawn(TrafficCone)
        .insert(Transform::from_xyz(0.0, 1.0, 0.0));

    commands
        .spawn(GasCan)
        .insert(Transform::from_xyz(5.0, 1.0, 0.0));

    commands
        .spawn(Fireball)
        .insert(Transform::from_xyz(0.0, 1.0, -3.0));

    commands
        .spawn(Block)
        .insert(Transform::from_xyz(-4.0, 3.0, 24.0));

    commands
        .spawn(Block)
        .insert(Transform::from_xyz(4.0, 3.0, 16.0));

    commands
        .spawn(Block)
        .insert(Transform::from_xyz(4.0, 3.0, 24.0));

    commands
        .spawn(Block)
        .insert(Transform::from_xyz(0.0, 2.0, 20.0));

    commands
        .spawn(Block)
        .insert(Transform::from_xyz(-20.0, 1.0, 2.0));

    commands
        .spawn(Block)
        .insert(Transform::from_xyz(-22.0, 1.0, 2.0));

    commands
        .spawn(Block)
        .insert(Transform::from_xyz(-22.0, 1.0, 4.0));
}
