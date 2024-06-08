use bevy_rapier3d::dynamics::{RigidBody, Velocity};

use bevy_rapier3d::prelude::Collider;

use bevy::prelude::*;

#[derive(Bundle)]
pub struct Package {
    name: Name,
    transform: TransformBundle,
    collider: Collider,
    velocity: Velocity,
    body: RigidBody,
}

impl Package {
    pub fn new() -> Self {
        Self {
            name: Name::new("Package"),
            transform: TransformBundle::default(),
            collider: Collider::cuboid(0.5, 0.5, 0.5),
            velocity: Velocity::default(),
            body: RigidBody::Dynamic,
        }
    }
}
