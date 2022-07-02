use crate::components::physics::Physics;
use bevy::{ecs::query, prelude::*};

pub struct PhysicsPlugin();

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(Self::movement);
    }
}

impl PhysicsPlugin {
    fn movement(mut entity: Query<(&mut Transform, &mut Physics)>, time: Res<Time>) {
        for (mut transform, mut physics) in entity.iter_mut() {
            physics.impulse += Vec3::new(0.0, -0.5, 0.0);
            transform.translation += physics.impulse * time.delta_seconds();
        }
    }
}
