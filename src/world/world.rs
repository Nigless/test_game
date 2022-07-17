use std::{collections::HashMap, f32::consts::PI};

use super::{block::Block, chunk::Chunk, generator::Generator};
pub struct World {
    radius: usize,
    pub grid: HashMap<[isize; 3], Box<Chunk>>,
}

impl World {
    pub fn new(radius: usize) -> Self {
        let mut grid = HashMap::with_capacity(10 * 10 * 10);
        let generator = Generator::new();

        for chunk_x in -2..2 {
            for chunk_y in -2..2 {
                for chunk_z in -2..2 {
                    grid.insert(
                        [chunk_x, chunk_y, chunk_z],
                        Box::new(Chunk::new(&generator, chunk_x, chunk_y, chunk_z)),
                    );
                }
            }
        }
        Self { radius, grid }
    }

    pub fn set_radius(&mut self, radius: usize) {
        self.radius = radius
    }

    pub fn get_block(&self, x: isize, y: isize, z: isize) -> &Block {
        self.grid
            .get(&[
                x / Chunk::SIZE as isize,
                y / Chunk::SIZE as isize,
                z / Chunk::SIZE as isize,
            ])
            .unwrap()
            .get_block(
                (Chunk::SIZE as isize - x % Chunk::SIZE as isize) as usize,
                (Chunk::SIZE as isize - y % Chunk::SIZE as isize) as usize,
                (Chunk::SIZE as isize - z % Chunk::SIZE as isize) as usize,
            )
    }

    pub fn get_chunk(&self, chunk_x: isize, chunk_y: isize, chunk_z: isize) -> &Chunk {
        self.grid.get(&[chunk_x, chunk_y, chunk_z]).unwrap()
    }
}
