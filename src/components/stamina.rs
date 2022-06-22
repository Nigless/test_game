use bevy::ecs::component::Component;

#[derive(Component)]
pub struct Stamina {
    total: u16,
    max: u16,
}

impl Stamina {
    pub fn new(max: u16) -> Self {
        Self { total: max, max }
    }
}
