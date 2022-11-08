use crate::sprite::WithSprite;
use bevy::prelude::*;

#[derive(Bundle)]
pub struct BndSprite {
    sprite: WithSprite,
    visibility: Visibility,
    computed_visibility: ComputedVisibility,
}

impl BndSprite {
    pub fn new(src: &str, size: f32) -> Self {
        Self {
            sprite: WithSprite::new(src, size),
            visibility: Default::default(),
            computed_visibility: Default::default(),
        }
    }
}
