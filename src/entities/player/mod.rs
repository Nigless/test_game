pub mod moving;
use crate::components::stamina::Stamina;
use crate::head::WithHead;
use crate::{components::health::Health, head::Head};

use bevy::{
    gltf::{Gltf, GltfNode},
    prelude::*,
};
use bevy_rapier3d::prelude::{Collider, ExternalForce, LockedAxes, RigidBody, Velocity};
use moving::Moving;

#[derive(Bundle)]
pub struct Player {
    head: WithHead,
    health: Health,
    stamina: Stamina,
    transform: Transform,
    moving: Moving,

    rigid_body: RigidBody,
    collider: Collider,
    locked_axes: LockedAxes,
    velocity: Velocity,

    scene: Handle<Scene>,
    material: Handle<StandardMaterial>,
    global_transform: GlobalTransform,
    visibility: Visibility,
    computed_visibility: ComputedVisibility,
}

impl Player {
    pub fn new(server: Res<AssetServer>) -> Self {
        Self {
            health: Health::new(100),
            head: WithHead,
            stamina: Stamina::new(100),
            transform: Transform::from_xyz(0.0, 2.0, 0.0),
            moving: Moving::default(),

            rigid_body: RigidBody::Dynamic,
            collider: Collider::cylinder(0.95, 0.2),
            locked_axes: LockedAxes::ROTATION_LOCKED,
            velocity: Velocity::default(),

            scene: server.load("robot/model.glb#Scene0"),
            material: Default::default(),
            global_transform: Default::default(),
            visibility: Default::default(),
            computed_visibility: Default::default(),
        }
    }
}
