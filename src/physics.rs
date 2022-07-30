use bevy::prelude::*;

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

        let impulse = (Vec3::new(0.0, -1000.0, 0.0) - physics.impulse) / 1000.0;
        physics.impulse.y += impulse.y * 100.0 * time.delta_seconds();
    }
}

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

    pub fn mov_x(&mut self, value: f32) {
        self.impulse.x = self.calculate(Vec3::new(value, 0.0, 0.0)).x;
    }

    pub fn mov_y(&mut self, value: f32) {
        self.impulse.y = self.calculate(Vec3::new(0.0, value, 0.0)).y;
    }

    pub fn mov_z(&mut self, value: f32) {
        self.impulse.z = self.calculate(Vec3::new(0.0, 0.0, value)).z;
    }

    pub fn mov(&mut self, vec: Vec3) {
        self.impulse = self.calculate(vec);
    }

    fn calculate(&self, mut vec: Vec3) -> Vec3 {
        if self.mass != 0 {
            vec = self.impulse + (vec - self.impulse) / self.mass as f32;
        }

        vec
    }
}
