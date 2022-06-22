use bevy::ecs::component::Component;

#[derive(Component)]
pub struct Speed {
    pub value: u16,
}

impl Speed {
    pub fn new(value: u16) -> Self {
        Self { value }
    }
}
