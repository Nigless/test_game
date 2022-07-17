use super::world::World;
use super::{chunk::Chunk, generator::Generator};
use crate::components::physics::Physics;
use bevy::{ecs::query, prelude::*};
use noise::{Perlin, Seedable};

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
        let world = World::new(10);
        for chunk_x in -1..1 {
            for chunk_y in -1..1 {
                for chunk_z in -1..1 {
                    commands.spawn_bundle(PbrBundle {
                        mesh: meshes.add(world.get_chunk(chunk_x, chunk_y, chunk_z).build_mesh(
                            world.get_chunk(chunk_x + 1, chunk_y, chunk_z),
                            world.get_chunk(chunk_x - 1, chunk_y, chunk_z),
                            world.get_chunk(chunk_x, chunk_y + 1, chunk_z),
                            world.get_chunk(chunk_x, chunk_y - 1, chunk_z),
                            world.get_chunk(chunk_x, chunk_y, chunk_z + 1),
                            world.get_chunk(chunk_x, chunk_y, chunk_z - 1),
                        )),

                        material: materials.add(StandardMaterial {
                            base_color: Color::GRAY,
                            reflectance: 0.0,
                            unlit: false,
                            ..Default::default()
                        }),
                        transform: Transform::from_translation(Vec3::new(
                            chunk_x as f32 * Chunk::SIZE as f32,
                            chunk_y as f32 * Chunk::SIZE as f32,
                            chunk_z as f32 * Chunk::SIZE as f32,
                        )),
                        ..Default::default()
                    });
                }
            }
        }
    }
}
