use std::{f32::consts, process::Child};

use bevy::prelude::*;
use bevy_rapier3d::prelude::{Collider, ColliderMassProperties, RigidBody, Velocity};

use crate::{
    camera_controller::Spectate, control::Control, entities::ghost::GhostBundle, model::Model,
    with_child::WithChild, with_material::WithMaterial, with_mesh::WithMesh,
};

#[derive(Bundle)]
pub struct TestLevel {
    name: Name,
    model: Model,
    body: RigidBody,
    player: WithChild<Player>,
    light: WithChild<Light>,
    block: WithChild<Block>,
}

impl Default for TestLevel {
    fn default() -> Self {
        Self {
            name: Name::new("test_scene"),
            model: Model::new("test_scene.glb"),
            body: RigidBody::Fixed,
            player: default(),
            light: default(),
            block: default(),
        }
    }
}

#[derive(Bundle)]
struct Light {
    name: Name,
    directional_light: DirectionalLight,
    transform: Transform,
}

impl Default for Light {
    fn default() -> Self {
        Self {
            name: Name::new("directional_light"),
            directional_light: DirectionalLight {
                illuminance: 3000.0,
                shadows_enabled: true,
                color: Color::WHITE,
                ..default()
            },
            transform: Transform::from_rotation(
                Quat::from_rotation_y(consts::PI * -0.1) * Quat::from_rotation_x(consts::PI * -0.6),
            ),
        }
    }
}

#[derive(Bundle)]
struct Player {
    ghost: GhostBundle,
    transform: Transform,
    spectate: Spectate,
    control: Control,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            ghost: GhostBundle::new(),
            transform: Transform::from_xyz(0.0, 3.0, 0.0),
            spectate: Spectate,
            control: Control,
        }
    }
}

#[derive(Bundle)]
struct Block {
    name: Name,
    mesh: WithMesh,
    material: WithMaterial,
    collider: Collider,
    body: RigidBody,
    transform: Transform,
    velocity: Velocity,
    collider_mass_properties: ColliderMassProperties,
}

impl Default for Block {
    fn default() -> Self {
        Self {
            name: Name::new("block"),
            mesh: WithMesh::new(Cuboid::new(1.0, 1.0, 1.0)),
            material: WithMaterial::new(Color::srgb_u8(255, 255, 255)),
            collider: Collider::cuboid(0.5, 0.5, 0.5),
            body: RigidBody::Dynamic,
            transform: Transform::from_xyz(0.0, 10.0, 10.0),
            velocity: Velocity::default(),
            collider_mass_properties: ColliderMassProperties::Mass(1000.0),
        }
    }
}
