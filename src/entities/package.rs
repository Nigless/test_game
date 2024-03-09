use bevy_rapier3d::dynamics::{GravityScale, RigidBody, Velocity};

use bevy_rapier3d::prelude::Collider;

use bevy::prelude::*;

#[derive(Bundle)]
pub struct Package {
    transform: TransformBundle,
    collider: Collider,
    velocity: Velocity,
    body: RigidBody,
}

impl Package {
    pub fn new() -> Self {
        Self {
            transform: TransformBundle::default(),
            collider: Collider::cuboid(1.0, 1.0, 1.0),
            velocity: Velocity::default(),
            body: RigidBody::Dynamic,
        }
    }
}
