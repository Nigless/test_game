use noise::{NoiseFn, Perlin, Seedable};

use super::block::Block;

pub struct Generator {
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
            .get([x as f64 * 0.01, y as f64 * 0.01, z as f64 * 0.01])
            > 0.0
        {
            return Block { is_solid: true };
        }
        Block::default()
    }
}
