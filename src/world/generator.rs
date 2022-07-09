use noise::{NoiseFn, Perlin, Seedable};

use super::block::Block;

struct Generator {
    noise: Perlin,
}

impl Generator {
    pub fn new() -> Self {
        let noise = Perlin::new();
        noise.set_seed(484);
        Self { noise }
    }
    pub fn get(&self, x: isize, y: isize, z: isize) -> Block {
        if self
            .noise
            .get([x as f64 * 0.027, y as f64 * 0.027, z as f64 * 0.027])
            > 0.0
        {
            return Block::new();
        }
        Block::new()
    }
}
