use crate::components::health::Health;
use crate::components::name::Name;
use crate::components::physics::Physics;
use crate::components::stamina::Stamina;
use bevy::ecs::bundle::Bundle;
use bevy::prelude::Transform;
use std::borrow::Borrow;

use super::state::State;

#[derive(Bundle)]
pub struct Player {
    health: Health,
    stamina: Stamina,
    name: Name,
    physics: Physics,
    transform: Transform,
    state: State,
}

impl Player {
    pub fn new(name: String) -> Self {
        Self {
            health: Health::new(100),
            stamina: Stamina::new(100),
            name: Name::new(name),
            physics: Physics::new(5),
            transform: Transform::default(),
            state: State::default(),
        }
    }
}
