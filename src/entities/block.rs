use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{with_material::WithMaterial, with_mesh::WithMesh};

#[derive(Bundle)]
pub struct BlockBundle {
    name: Name,
    mesh: WithMesh,
    material: WithMaterial,
    collider: Collider,
    body: RigidBody,
    collider_mass_properties: ColliderMassProperties,
}

impl Default for BlockBundle {
    fn default() -> Self {
        Self {
            name: Name::new("block"),
            mesh: WithMesh::new(Cuboid::new(1.0, 1.0, 1.0)),
            material: WithMaterial::new(Color::srgb_u8(255, 255, 255)),
            collider: Collider::cuboid(0.5, 0.5, 0.5),
            body: RigidBody::Dynamic,
            collider_mass_properties: ColliderMassProperties::Mass(1.0),
        }
    }
}

impl BlockBundle {
    pub fn new(hx: f32, hy: f32, hz: f32) -> Self {
        Self {
            name: Name::new("block"),
            mesh: WithMesh::new(Cuboid::new(hx, hy, hz)),
            material: WithMaterial::new(Color::srgb_u8(255, 255, 255)),
            collider: Collider::cuboid(hx / 2.0, hy / 2.0, hz / 2.0),
            body: RigidBody::Dynamic,
            collider_mass_properties: ColliderMassProperties::Mass(1.0),
        }
    }
}
