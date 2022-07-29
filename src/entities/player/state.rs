use bevy::ecs::component::Component;

#[derive(Component, Default)]
pub struct State {
    pub ducking: bool,
    pub jumping: bool,
    pub moving: bool,
    pub pushing_down: bool,
    pub sliding: bool,
}
