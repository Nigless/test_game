use std::{thread, time::Duration};

use bevy::{app::Plugin, ecs::system::Resource, prelude::*};

#[derive(Resource, Reflect)]
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
                First,
                throttle.run_if(|throttle: Res<Throttle>| throttle.enabled),
            );
    }
}

fn throttle(throttle: Res<Throttle>) {
    thread::sleep(Duration::from_secs_f32(
        1.0 / (throttle.target_frame_rate as f32),
    ));
}
