use crate::components::physics::Physics;
use bevy::{ecs::query, prelude::*};
use noise::{Perlin, Seedable};

use super::chunk::Chunk;

pub struct WorldPlugin();

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(Self::startup);
    }
}

impl WorldPlugin {
    fn startup(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        let noise = Perlin::new();
        noise.set_seed(484);

        commands.spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(Chunk::new(&noise, 0, 0, 0))),
            material: materials.add(StandardMaterial {
                base_color: Color::GOLD,
                reflectance: 0.0,
                unlit: false,
                ..Default::default()
            }),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..Default::default()
        });
        commands.spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(Chunk::new(&noise, 1, 0, 0))),
            material: materials.add(StandardMaterial {
                base_color: Color::GOLD,
                reflectance: 0.0,
                unlit: false,
                ..Default::default()
            }),
            transform: Transform::from_translation(Vec3::new(32.0, 0.0, 0.0)),
            ..Default::default()
        });
        commands.spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(Chunk::new(&noise, -1, 0, 0))),
            material: materials.add(StandardMaterial {
                base_color: Color::GOLD,
                reflectance: 0.0,
                unlit: false,
                ..Default::default()
            }),
            transform: Transform::from_translation(Vec3::new(-32.0, 0.0, 0.0)),
            ..Default::default()
        });
    }
}
