use bevy::ecs::component::Component;

#[derive(Component)]
pub struct Health {
    total: u16,
    max: u16,
}
impl Health {
    pub fn new(max: u16) -> Self {
        Self { total: max, max }
    }
}
