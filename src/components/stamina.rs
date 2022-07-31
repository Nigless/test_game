use bevy::prelude::Component;

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
