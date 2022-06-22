use std::borrow::Borrow;

use crate::components::health::Health;
use crate::components::name::Name;
use crate::components::speed::Speed;
use crate::components::stamina::Stamina;
use bevy::ecs::bundle::Bundle;
use bevy::prelude::Transform;

#[derive(Bundle)]
pub struct Player {
    health: Health,
    stamina: Stamina,
    name: Name,
    speed: Speed,
    transform: Transform,
}

impl Player {
    pub fn new(name: String) -> Self {
        Self {
            health: Health::new(100),
            stamina: Stamina::new(100),
            name: Name::new(name),
            speed: Speed::new(10),
            transform: Transform::default(),
        }
    }
}
