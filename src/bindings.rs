use bevy::{
    ecs::{component::Component, system::Resource},
    input::keyboard::KeyCode,
};

#[derive(Component)]
pub struct Control;

#[derive(Resource)]
pub struct Bindings {
    pub move_forward: KeyCode,
    pub move_left: KeyCode,
    pub move_right: KeyCode,
    pub move_backward: KeyCode,
    pub jump: KeyCode,
    pub crouch: KeyCode,
    pub run: KeyCode,
    pub pause: KeyCode,
}

impl Default for Bindings {
    fn default() -> Self {
        Self {
            move_forward: KeyCode::KeyW,
            move_left: KeyCode::KeyA,
            move_right: KeyCode::KeyD,
            move_backward: KeyCode::KeyS,
            jump: KeyCode::Space,
            crouch: KeyCode::ControlLeft,
            run: KeyCode::ShiftLeft,
            pause: KeyCode::Escape,
        }
    }
}
