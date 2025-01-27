use bevy::{prelude::Component, reflect::Reflect};

use crate::with_child::WithChild;
use bevy::prelude::*;
use bevy_rapier3d::prelude::{GravityScale, Velocity};

use super::{Player, COLLIDER_HALF_HEIGHT};

#[derive(Component, Reflect, PartialEq)]
#[reflect(Component)]
#[require(GravityScale, Status, Velocity)]
pub struct Parameters {
    pub walking_speed: f32,
    pub falling_speed: f32,
    pub running_speed: f32,
    pub crouching_speed: f32,
    pub swimming_speed: f32,
    pub falling_acceleration: f32,
    pub standing_acceleration: f32,
    pub standing_jump_height: f32,
    pub crouching_jump_height: f32,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            walking_speed: 4.0,
            running_speed: 8.0,
            falling_speed: 2.0,
            falling_acceleration: 3.0,
            standing_acceleration: 40.0,
            standing_jump_height: 4.0,
            crouching_jump_height: 2.0,
            crouching_speed: 2.0,
            swimming_speed: 60.0,
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Status {
    pub surface: Option<Vec3>,
    pub current_collider_height: f32,
    pub can_standup: bool,
}

impl Default for Status {
    fn default() -> Self {
        Self {
            surface: None,
            current_collider_height: COLLIDER_HALF_HEIGHT,
            can_standup: true,
        }
    }
}
