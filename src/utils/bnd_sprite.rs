use crate::sprite::WithSprite;
use bevy::prelude::*;
use bevy_rapier3d::prelude::LockedAxes;

#[derive(Bundle)]
pub struct BndSprite {
    sprite: WithSprite,
    locked_axes: LockedAxes,
    visibility: Visibility,
    computed_visibility: ComputedVisibility,
}

impl BndSprite {
    pub fn new(src: &str) -> Self {
        Self {
            sprite: WithSprite::new(src),
            locked_axes: LockedAxes::ROTATION_LOCKED,
            visibility: Default::default(),
            computed_visibility: Default::default(),
        }
    }
}
