use bevy::{ecs::component::Component, math::Vec3};

#[derive(Component)]
pub struct Physics {
    pub mass: u16,
    pub impulse: Vec3,
}

impl Physics {
    pub fn new(mass: u16) -> Self {
        Self {
            mass,
            impulse: Vec3::ZERO,
        }
    }

    pub fn mov(&mut self, vec: Vec3) {
        if self.mass == 0 {
            self.impulse = vec;

            return;
        }
        self.impulse = self.impulse + (vec - self.impulse) / self.mass as f32;
    }
}
