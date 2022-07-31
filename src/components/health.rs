use bevy::prelude::Component;

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
