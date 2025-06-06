use std::default;

use bevy::prelude::*;

#[derive(Reflect, Default)]
pub struct DevSettings {
    pub show_explosion_hits: bool,
    pub show_colliders: bool,
    pub show_ray_casters: bool,
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct Settings {
    pub mouse_sensitivity: f32,
    pub dev_settings: DevSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            mouse_sensitivity: 0.001,
            dev_settings: default(),
        }
    }
}

impl Settings {
    pub fn is<C: Fn(&Settings) -> bool>(condition: C) -> impl Fn(Res<Settings>) -> bool {
        move |settings: Res<Settings>| condition(&settings)
    }
}

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.register_type::<Settings>()
            .insert_resource(Settings::default());
    }
}
