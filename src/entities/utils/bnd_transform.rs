use bevy::prelude::*;

#[derive(Bundle, Default)]
pub struct BndTransform {
    transform: Transform,
    global_transform: GlobalTransform,
}

impl BndTransform {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            transform: Transform::from_xyz(x, y, z),
            global_transform: GlobalTransform::default(),
        }
    }
}
