use super::generator::Generator;
use super::world::World;
use super::{block::Block, side::Side};
use bevy::{
    prelude::Mesh,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_resource::std430::Vec3,
    },
};
use noise::{NoiseFn, Perlin, Seedable};

pub struct Chunk {
    grid: [Block; Self::SIZE * Self::SIZE * Self::SIZE],
}

impl Chunk {
    pub const SIZE: usize = 64;

    pub fn new(generator: &Generator, chunk_x: isize, chunk_y: isize, chunk_z: isize) -> Self {
        let mut grid = [Block::default(); Self::SIZE * Self::SIZE * Self::SIZE];
        let size = grid.len();
        for i in 0..size {
            let local_x = (i as f32 / (Self::SIZE * Self::SIZE) as f32) as isize;
            let local_y = ((i as f32 / Self::SIZE as f32) % Self::SIZE as f32) as isize;
            let local_z = (i as f32 % Self::SIZE as f32) as isize;

            grid[i] = generator.get(
                local_x + chunk_x * Self::SIZE as isize,
                local_y + chunk_y * Self::SIZE as isize,
                local_z + chunk_z * Self::SIZE as isize,
            )
        }

        Self { grid }
    }

    pub fn get_block(&self, local_x: usize, local_y: usize, local_z: usize) -> &Block {
        &self.grid[local_z + local_y * Self::SIZE + local_x * Self::SIZE * Self::SIZE]
    }

    pub fn build_mesh(
        &self,
        front: &Self,
        back: &Self,
        top: &Self,
        bottom: &Self,
        left: &Self,
        right: &Self,
    ) -> Mesh {
        let get_block = |local_x, local_y, local_z| -> &Block {
            if local_x >= Self::SIZE as isize {
                return front.get_block(0, local_y as usize, local_z as usize);
            }
            if local_x < 0 {
                return back.get_block(Self::SIZE - 1, local_y as usize, local_z as usize);
            }
            if local_y >= Self::SIZE as isize {
                return top.get_block(local_x as usize, 0, local_z as usize);
            }
            if local_y < 0 {
                return bottom.get_block(local_x as usize, Self::SIZE - 1, local_z as usize);
            }
            if local_z >= Self::SIZE as isize {
                return left.get_block(local_x as usize, local_y as usize, 0);
            }
            if local_z < 0 {
                return right.get_block(local_x as usize, local_y as usize, Self::SIZE - 1);
            }
            self.get_block(local_x as usize, local_y as usize, local_z as usize)
        };

        let mut positions: Vec<[f32; 3]> = vec![];
        let mut normals: Vec<[f32; 3]> = vec![];
        let mut uvs: Vec<[f32; 2]> = vec![];

        for local_x in 0..Self::SIZE {
            for local_y in 0..Self::SIZE {
                for local_z in 0..Self::SIZE {
                    if !self.get_block(local_x, local_y, local_z).is_solid() {
                        continue;
                    }

                    if !get_block(local_x as isize + 1, local_y as isize, local_z as isize)
                        .is_solid()
                    {
                        Side::Front
                            .to_positions(local_x, local_y, local_z)
                            .map(|p| positions.push(p));
                        Side::Front.to_normals().map(|n| normals.push(n));
                        Side::to_uvs(1.0, 0.0).map(|u| uvs.push(u));
                    }
                    if !get_block(local_x as isize - 1, local_y as isize, local_z as isize)
                        .is_solid()
                    {
                        Side::Back
                            .to_positions(local_x, local_y, local_z)
                            .map(|p| positions.push(p));
                        Side::Back.to_normals().map(|n| normals.push(n));
                        Side::to_uvs(1.0, 0.0).map(|u| uvs.push(u));
                    }

                    if !get_block(local_x as isize, local_y as isize + 1, local_z as isize)
                        .is_solid()
                    {
                        Side::Top
                            .to_positions(local_x, local_y, local_z)
                            .map(|p| positions.push(p));
                        Side::Top.to_normals().map(|n| normals.push(n));
                        Side::to_uvs(1.0, 0.0).map(|u| uvs.push(u));
                    }
                    if !get_block(local_x as isize, local_y as isize - 1, local_z as isize)
                        .is_solid()
                    {
                        Side::Bottom
                            .to_positions(local_x, local_y, local_z)
                            .map(|p| positions.push(p));
                        Side::Bottom.to_normals().map(|n| normals.push(n));
                        Side::to_uvs(1.0, 0.0).map(|u| uvs.push(u));
                    }

                    if !get_block(local_x as isize, local_y as isize, local_z as isize + 1)
                        .is_solid()
                    {
                        Side::Left
                            .to_positions(local_x, local_y, local_z)
                            .map(|p| positions.push(p));
                        Side::Left.to_normals().map(|n| normals.push(n));
                        Side::to_uvs(1.0, 0.0).map(|u| uvs.push(u));
                    }
                    if !get_block(local_x as isize, local_y as isize, local_z as isize - 1)
                        .is_solid()
                    {
                        Side::Right
                            .to_positions(local_x, local_y, local_z)
                            .map(|p| positions.push(p));
                        Side::Right.to_normals().map(|n| normals.push(n));
                        Side::to_uvs(1.0, 0.0).map(|u| uvs.push(u));
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
