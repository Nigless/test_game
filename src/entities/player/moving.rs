use bevy::prelude::*;

#[derive(Component, Default)]
pub struct Moving {
    pub ducking: bool,
    pub jumping: bool,
    pub moving: bool,
    pub pushing_down: bool,
    pub sliding: bool,
}
