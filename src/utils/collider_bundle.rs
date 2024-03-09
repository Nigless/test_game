use bevy::prelude::*;
use bevy_rapier3d::prelude::{Collider, RigidBody, Velocity};

#[derive(Bundle, Default)]
pub struct ColliderBundle {
    rigid_body: RigidBody,
    collider: Collider,
    velocity: Velocity,
}

impl ColliderBundle {
    pub fn new(collider: Collider) -> Self {
        Self {
            rigid_body: RigidBody::KinematicVelocityBased,
            collider,
            velocity: Velocity::default(),
        }
    }

    pub fn default() -> Self {
        Self {
            rigid_body: RigidBody::KinematicVelocityBased,
            collider: Collider::ball(1.0),
            velocity: Velocity::default(),
        }
    }
}
