use bevy::prelude::*;
use bevy_rapier3d::prelude::{Collider, RigidBody, Velocity};

#[derive(Bundle, Default)]
pub struct BndCollider {
    rigid_body: RigidBody,
    collider: Collider,
    velocity: Velocity,
}

impl BndCollider {
    pub fn new(collider: Collider) -> Self {
        Self {
            rigid_body: RigidBody::Dynamic,
            collider,
            velocity: Velocity::default(),
        }
    }

    pub fn default() -> Self {
        Self {
            rigid_body: RigidBody::Dynamic,
            collider: Collider::ball(1.0),
            velocity: Velocity::default(),
        }
    }
}
