use bevy::prelude::*;
use smallstr::SmallString;

#[derive(Component)]
pub struct Name {
    pub value: SmallString<[u8; 32]>,
}

impl Name {
    pub fn new(value: String) -> Self {
        Self {
            value: SmallString::from(value),
        }
    }
}

#[derive(Component)]
pub struct Health {
    pub total: u16,
    pub max: u16,
}

impl Health {
    pub fn new(max: u16) -> Self {
        Self { total: max, max }
    }
}

#[derive(Component)]
pub struct Stamina {
    pub total: u16,
    pub max: u16,
}

impl Stamina {
    pub fn new(max: u16) -> Self {
        Self { total: max, max }
    }
}

#[derive(Component, Default)]
pub struct Character {
    pub ducking: bool,
    pub jumping: bool,
    pub moving: bool,
    pub pushing_down: bool,
    pub sliding: bool,
}
