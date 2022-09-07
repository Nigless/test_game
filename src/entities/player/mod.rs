pub mod moving;
use crate::components::health::Health;
use crate::components::name::Name;
use crate::components::stamina::Stamina;
use bevy::{gltf::GltfNode, prelude::*};
use bevy_rapier3d::prelude::{Collider, ExternalForce, LockedAxes, RigidBody, Velocity};
use moving::Moving;

#[derive(Bundle)]
pub struct Player {
    name: Name,
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
    pub fn new(name: String, server: Res<AssetServer>) -> Self {
        Self {
            health: Health::new(100),
            stamina: Stamina::new(100),
            name: Name::new(name),
            transform: Transform::from_xyz(0.0, 2.0, 0.0),
            moving: Moving::default(),

            rigid_body: RigidBody::Dynamic,
            collider: Collider::cylinder(0.8, 0.2),
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
