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

    fn calculate(&self, mut vec: Vec3) -> Vec3 {
        if self.mass != 0 {
            vec = self.impulse + (vec - self.impulse) / self.mass as f32;
        }

        vec
    }

    pub fn mov(&mut self, vec: Vec3) {
        self.impulse = self.calculate(vec);
    }

    pub fn mov_x(&mut self, value: f32) {
        self.impulse.x = self.calculate(Vec3::new(value, 0.0, 0.0)).x;
    }

    pub fn mov_y(&mut self, value: f32) {
        self.impulse.y = self.calculate(Vec3::new(0.0, value, 0.0)).y;
    }

    pub fn mov_z(&mut self, value: f32) {
        self.impulse.z = self.calculate(Vec3::new(0.0, 0.0, value)).z;
    }
}
