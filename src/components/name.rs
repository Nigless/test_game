use bevy::ecs::component::Component;
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
