use crate::components::health::Health;
use crate::components::name::Name;
use crate::components::physics::Physics;
use crate::components::stamina::Stamina;
use bevy::ecs::bundle::Bundle;
use bevy::prelude::Transform;
use std::borrow::Borrow;

#[derive(Bundle)]
pub struct Player {
    health: Health,
    stamina: Stamina,
    name: Name,
    physics: Physics,
    transform: Transform,
}

impl Player {
    pub fn new(name: String) -> Self {
        Self {
            health: Health::new(100),
            stamina: Stamina::new(100),
            name: Name::new(name),
            physics: Physics::new(0),
            transform: Transform::default(),
        }
    }
}
