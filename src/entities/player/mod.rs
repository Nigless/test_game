pub mod moving;
use crate::components::health::Health;
use crate::components::name::Name;
use crate::components::stamina::Stamina;
use crate::control::Control;
use crate::physics::Physics;
use bevy::prelude::*;
use moving::Moving;

#[derive(Bundle)]
pub struct Player {
    pub name: Name,
    pub health: Health,
    pub stamina: Stamina,
    pub physics: Physics,
    pub transform: Transform,
    pub moving: Moving,
}

impl Player {
    pub fn new(name: String) -> Self {
        Self {
            health: Health::new(100),
            stamina: Stamina::new(100),
            name: Name::new(name),
            physics: Physics::new(5),
            transform: Transform::default(),
            moving: Moving::default(),
        }
    }
}
