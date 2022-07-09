use super::block::Block;
use bevy::{
    prelude::Mesh,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_resource::std430::Vec3,
    },
};
use noise::{NoiseFn, Perlin, Seedable};

const CHUNK_SIZE: usize = 32;
type Grid = [[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];

pub struct Chunk {
    grid: Grid,
}

impl Chunk {
    pub fn new(noise: &Perlin, chunk_x: isize, chunk_y: isize, chunk_z: isize) -> Self {
        let mut grid = Grid::default();
        let size = grid.len();
        for x in 0..size {
            for y in 0..size {
                for z in 0..size {
                    if noise.get([
                        (x as isize + chunk_x * size as isize) as f64 * 0.027,
                        (y as isize + chunk_y * size as isize) as f64 * 0.027,
                        (z as isize + chunk_z * size as isize) as f64 * 0.027,
                    ]) > 0.0
                    {
                        grid[x][y][z] = Block { is_solid: true };
                    }
                }
            }
        }

        Self { grid }
    }

    pub fn get_size(&self) -> usize {
        return self.grid.len();
    }

    pub fn get_block(&self, x: usize, y: usize, z: usize) -> &Block {
        &self.grid[x][y][z]
    }

    // pub fn generate(&self, x: isize, y: isize, z: isize) -> bool {
    //     self.noise.get([
    //         (x as isize + chunk_x * size as isize) as f64 * 0.027,
    //         (y as isize + chunk_y * size as isize) as f64 * 0.027,
    //         (z as isize + chunk_z * size as isize) as f64 * 0.027,
    //     ]) > 0.0
    // }
}

impl From<Chunk> for Mesh {
    fn from(chunk: Chunk) -> Self {
        let mut positions: Vec<[f32; 3]> = vec![];
        let mut normals: Vec<[f32; 3]> = vec![];
        let mut uvs: Vec<[f32; 2]> = vec![];
        let size = chunk.get_size();
        // Top
        // Bottom
        // Right
        // Left
        // Front
        // Back

        for x in 0..size {
            for y in 0..size {
                for z in 0..size {
                    if !chunk.get_block(x, y, z).is_solid() {
                        continue;
                    }

                    if x == size - 1 || !chunk.get_block(x + 1, y, z).is_solid() {
                        [
                            [0.5 + x as f32, -0.5 + y as f32, 0.5 + z as f32],
                            [0.5 + x as f32, -0.5 + y as f32, -0.5 + z as f32],
                            [0.5 + x as f32, 0.5 + y as f32, -0.5 + z as f32],
                            [0.5 + x as f32, -0.5 + y as f32, 0.5 + z as f32],
                            [0.5 + x as f32, 0.5 + y as f32, -0.5 + z as f32],
                            [0.5 + x as f32, 0.5 + y as f32, 0.5 + z as f32],
                        ]
                        .map(|value| positions.push(value));
                        [
                            [1.0, 0.0, 0.0],
                            [1.0, 0.0, 0.0],
                            [1.0, 0.0, 0.0],
                            [1.0, 0.0, 0.0],
                            [1.0, 0.0, 0.0],
                            [1.0, 0.0, 0.0],
                        ]
                        .map(|value| normals.push(value));
                        [
                            [0.0, 1.0],
                            [0.0, 0.0],
                            [1.0, 0.0],
                            [0.0, 1.0],
                            [1.0, 0.0],
                            [1.0, 1.0],
                        ]
                        .map(|value| uvs.push(value));
                    }
                    if x == 0 || !chunk.get_block(x - 1, y, z).is_solid() {
                        [
                            [-0.5 + x as f32, 0.5 + y as f32, 0.5 + z as f32],
                            [-0.5 + x as f32, 0.5 + y as f32, -0.5 + z as f32],
                            [-0.5 + x as f32, -0.5 + y as f32, 0.5 + z as f32],
                            [-0.5 + x as f32, 0.5 + y as f32, -0.5 + z as f32],
                            [-0.5 + x as f32, -0.5 + y as f32, -0.5 + z as f32],
                            [-0.5 + x as f32, -0.5 + y as f32, 0.5 + z as f32],
                        ]
                        .map(|value| positions.push(value));
                        [
                            [-1.0, 0.0, 0.0],
                            [-1.0, 0.0, 0.0],
                            [-1.0, 0.0, 0.0],
                            [-1.0, 0.0, 0.0],
                            [-1.0, 0.0, 0.0],
                            [-1.0, 0.0, 0.0],
                        ]
                        .map(|value| normals.push(value));
                        [
                            [0.0, 1.0],
                            [0.0, 0.0],
                            [1.0, 0.0],
                            [0.0, 1.0],
                            [1.0, 0.0],
                            [1.0, 1.0],
                        ]
                        .map(|value| uvs.push(value));
                    }

                    if y == size - 1 || !chunk.get_block(x, y + 1, z).is_solid() {
                        [
                            [0.5 + x as f32, 0.5 + y as f32, 0.5 + z as f32],
                            [0.5 + x as f32, 0.5 + y as f32, -0.5 + z as f32],
                            [-0.5 + x as f32, 0.5 + y as f32, 0.5 + z as f32],
                            [0.5 + x as f32, 0.5 + y as f32, -0.5 + z as f32],
                            [-0.5 + x as f32, 0.5 + y as f32, -0.5 + z as f32],
                            [-0.5 + x as f32, 0.5 + y as f32, 0.5 + z as f32],
                        ]
                        .map(|value| positions.push(value));
                        [
                            [0.0, 1.0, 0.0],
                            [0.0, 1.0, 0.0],
                            [0.0, 1.0, 0.0],
                            [0.0, 1.0, 0.0],
                            [0.0, 1.0, 0.0],
                            [0.0, 1.0, 0.0],
                        ]
                        .map(|value| normals.push(value));
                        [
                            [0.0, 1.0],
                            [0.0, 0.0],
                            [1.0, 0.0],
                            [0.0, 1.0],
                            [1.0, 0.0],
                            [1.0, 1.0],
                        ]
                        .map(|value| uvs.push(value));
                    }
                    if y == 0 || !chunk.get_block(x, y - 1, z).is_solid() {
                        [
                            [-0.5 + x as f32, -0.5 + y as f32, 0.5 + z as f32],
                            [-0.5 + x as f32, -0.5 + y as f32, -0.5 + z as f32],
                            [0.5 + x as f32, -0.5 + y as f32, -0.5 + z as f32],
                            [-0.5 + x as f32, -0.5 + y as f32, 0.5 + z as f32],
                            [0.5 + x as f32, -0.5 + y as f32, -0.5 + z as f32],
                            [0.5 + x as f32, -0.5 + y as f32, 0.5 + z as f32],
                        ]
                        .map(|value| positions.push(value));
                        [
                            [0.0, -1.0, 0.0],
                            [0.0, -1.0, 0.0],
                            [0.0, -1.0, 0.0],
                            [0.0, -1.0, 0.0],
                            [0.0, -1.0, 0.0],
                            [0.0, -1.0, 0.0],
                        ]
                        .map(|value| normals.push(value));
                        [
                            [0.0, 1.0],
                            [0.0, 0.0],
                            [1.0, 0.0],
                            [0.0, 1.0],
                            [1.0, 0.0],
                            [1.0, 1.0],
                        ]
                        .map(|value| uvs.push(value));
                    }

                    if z == size - 1 || !chunk.get_block(x, y, z + 1).is_solid() {
                        [
                            [-0.5 + x as f32, 0.5 + y as f32, 0.5 + z as f32],
                            [-0.5 + x as f32, -0.5 + y as f32, 0.5 + z as f32],
                            [0.5 + x as f32, -0.5 + y as f32, 0.5 + z as f32],
                            [-0.5 + x as f32, 0.5 + y as f32, 0.5 + z as f32],
                            [0.5 + x as f32, -0.5 + y as f32, 0.5 + z as f32],
                            [0.5 + x as f32, 0.5 + y as f32, 0.5 + z as f32],
                        ]
                        .map(|value| positions.push(value));
                        [
                            [0.0, 0.0, 1.0],
                            [0.0, 0.0, 1.0],
                            [0.0, 0.0, 1.0],
                            [0.0, 0.0, 1.0],
                            [0.0, 0.0, 1.0],
                            [0.0, 0.0, 1.0],
                        ]
                        .map(|value| normals.push(value));
                        [
                            [0.0, 1.0],
                            [0.0, 0.0],
                            [1.0, 0.0],
                            [0.0, 1.0],
                            [1.0, 0.0],
                            [1.0, 1.0],
                        ]
                        .map(|value| uvs.push(value));
                    }
                    if z == 0 || !chunk.get_block(x, y, z - 1).is_solid() {
                        [
                            [0.5 + x as f32, 0.5 + y as f32, -0.5 + z as f32],
                            [0.5 + x as f32, -0.5 + y as f32, -0.5 + z as f32],
                            [-0.5 + x as f32, 0.5 + y as f32, -0.5 + z as f32],
                            [0.5 + x as f32, -0.5 + y as f32, -0.5 + z as f32],
                            [-0.5 + x as f32, -0.5 + y as f32, -0.5 + z as f32],
                            [-0.5 + x as f32, 0.5 + y as f32, -0.5 + z as f32],
                        ]
                        .map(|value| positions.push(value));
                        [
                            [0.0, 0.0, -1.0],
                            [0.0, 0.0, -1.0],
                            [0.0, 0.0, -1.0],
                            [0.0, 0.0, -1.0],
                            [0.0, 0.0, -1.0],
                            [0.0, 0.0, -1.0],
                        ]
                        .map(|value| normals.push(value));
                        [
                            [0.0, 1.0],
                            [0.0, 0.0],
                            [1.0, 0.0],
                            [0.0, 1.0],
                            [1.0, 0.0],
                            [1.0, 1.0],
                        ]
                        .map(|value| uvs.push(value));
                    }
                }
            }
        }
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh
    }
}
