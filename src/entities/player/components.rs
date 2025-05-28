use bevy::{prelude::Component, reflect::Reflect};

use bevy::prelude::*;
use bevy_rapier3d::prelude::{GravityScale, Velocity};

use super::COLLIDER_HALF_HEIGHT;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct State {
    pub surface: Option<Vec3>,
    pub current_collider_height: f32,
    pub can_standup: bool,
    pub head_tilt: f32,
}

impl Default for State {
    fn default() -> Self {
        Self {
            surface: None,
            current_collider_height: COLLIDER_HALF_HEIGHT,
            can_standup: true,
            head_tilt: 0.0,
        }
    }
}
