use crate::character::{Character, Health, Name, Stamina};
use crate::control::Control;
use crate::physics::Physics;
use bevy::prelude::*;

#[derive(Bundle)]
pub struct Player {
    pub name: Name,
    pub health: Health,
    pub stamina: Stamina,
    pub physics: Physics,
    pub transform: Transform,
    pub character: Character,
    pub control: Control,
}

impl Player {
    pub fn new(name: String) -> Self {
        Self {
            health: Health::new(100),
            stamina: Stamina::new(100),
            name: Name::new(name),
            physics: Physics::new(5),
            transform: Transform::default(),
            character: Character::default(),
            control: Control,
        }
    }
}
