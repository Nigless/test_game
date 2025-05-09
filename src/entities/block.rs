use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{library::Spawnable, with_material::WithMaterial, with_mesh::WithMesh};

#[derive(Bundle, Clone)]
pub struct BlockBundle {
    name: Name,
    mesh: WithMesh,
    material: WithMaterial,
    collider: Collider,
    body: RigidBody,
    collider_mass_properties: ColliderMassProperties,
    velocity: Velocity,
    transform: Transform,
}

impl Default for BlockBundle {
    fn default() -> Self {
        Self {
            name: Name::new("block"),
            mesh: WithMesh::new(Cuboid::new(1.0, 1.0, 1.0)),
            material: WithMaterial::new(Color::srgb_u8(255, 255, 255)),
            collider: Collider::cuboid(0.5, 0.5, 0.5),
            body: RigidBody::Dynamic,
            collider_mass_properties: ColliderMassProperties::Mass(200.0),
            velocity: Velocity::default(),
            transform: Transform::default(),
        }
    }
}

impl BlockBundle {
    pub fn new(hx: f32, hy: f32, hz: f32) -> Self {
        Self {
            name: Name::new("block"),
            mesh: WithMesh::new(Cuboid::new(hx, hy, hz)),
            collider: Collider::cuboid(hx / 2.0, hy / 2.0, hz / 2.0),
            ..default()
        }
    }

    pub fn with_mass(mut self, mass: f32) -> Self {
        self.collider_mass_properties = ColliderMassProperties::Mass(mass);
        self
    }

    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }
}

impl Spawnable for BlockBundle {
    fn spawn<'a>(&self, commands: &'a mut Commands) -> EntityCommands<'a> {
        commands.spawn(self.clone())
    }
}
