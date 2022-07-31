use bevy::prelude::*;

#[derive(Component)]
pub struct Physics {
    pub mass: u16,
    pub impulse: Vec3,
    pub gravity: bool,
}

impl Physics {
    pub fn new(mass: u16) -> Self {
        Self {
            mass,
            impulse: Vec3::ZERO,
            gravity: true,
        }
    }

    pub fn mov_x(&mut self, value: f32) {
        self.impulse.x = self.calculate(self.impulse.x, value);
    }

    pub fn mov_y(&mut self, value: f32) {
        self.impulse.y = self.calculate(self.impulse.y, value);
    }

    pub fn mov_z(&mut self, value: f32) {
        self.impulse.z = self.calculate(self.impulse.z, value);
    }

    pub fn mov(&mut self, vec: Vec3) {
        self.mov_x(vec.x);
        self.mov_y(vec.y);
        self.mov_z(vec.z);
    }

    fn calculate(&self, impulse: f32, mut value: f32) -> f32 {
        if self.mass != 0 {
            value = impulse + (value - impulse) / self.mass as f32;
        }

        value
    }
}

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(movement);
    }
}

pub fn movement(mut entity: Query<(&mut Transform, &mut Physics)>, time: Res<Time>) {
    for (mut transform, mut physics) in entity.iter_mut() {
        transform.translation += physics.impulse * time.delta_seconds();

        // floor
        if transform.translation.y <= 0.0 {
            transform.translation.y = 0.0;
            physics.impulse.y = 0.0;
            return;
        }

        if physics.gravity {
            let impulse = (Vec3::new(0.0, -1000.0, 0.0) - physics.impulse) / 1000.0;
            physics.impulse.y += impulse.y * 100.0 * time.delta_seconds();
        }
    }
}
