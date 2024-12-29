use std::{thread::sleep, time::Duration};

use bevy::{
    app::Plugin,
    ecs::{component::Component, system::Resource},
    input::{keyboard::KeyCode, mouse::MouseMotion},
    math::Vec2,
    prelude::*,
};

#[derive(Resource, Reflect, PartialEq)]
#[reflect(Resource)]
pub struct Throttle {
    pub enabled: bool,
    pub target_frame_rate: u16,
}

impl Default for Throttle {
    fn default() -> Self {
        Self {
            enabled: false,
            target_frame_rate: 30,
        }
    }
}

pub struct ThrottlePlugin;

impl Plugin for ThrottlePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Throttle>()
            .insert_resource(Throttle::default())
            .add_systems(
                PreUpdate,
                throttle.run_if(|throttle: Res<Throttle>| throttle.enabled),
            );
    }
}

fn throttle(throttle: Res<Throttle>) {
    sleep(Duration::from_secs_f32(
        1.0 / (throttle.target_frame_rate as f32),
    ));
}
