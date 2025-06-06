use bevy::{
    app::Plugin,
    ecs::{component::Component, system::Resource},
    input::{keyboard::KeyCode, mouse::MouseMotion},
    math::Vec2,
    prelude::*,
};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

#[derive(Resource, PartialEq, Clone)]
pub struct Random {
    pub rand: ChaCha8Rng,
}

impl Default for Random {
    fn default() -> Self {
        Self {
            rand: ChaCha8Rng::from_entropy(),
        }
    }
}

impl Random {
    fn new(seed: u64) -> Self {
        Self {
            rand: ChaCha8Rng::seed_from_u64(seed),
        }
    }
}

pub struct RandomPlugin {
    random: Option<Random>,
}

impl Default for RandomPlugin {
    fn default() -> Self {
        Self {
            random: Some(Random::default()),
        }
    }
}

impl RandomPlugin {
    fn new(seed: u64) -> Self {
        Self {
            random: Some(Random::new(seed)),
        }
    }
}

impl Plugin for RandomPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.random.clone().unwrap_or_default());
    }
}
