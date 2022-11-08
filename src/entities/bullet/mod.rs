use crate::utils::bnd_collider::BndCollider;
use crate::utils::bnd_sprite::BndSprite;
use crate::utils::bnd_transform::BndTransform;
use bevy_rapier3d::prelude::*;

use bevy::prelude::*;

#[derive(Bundle)]
pub struct Bullet {
    gravity_scale: GravityScale,
    locked_axes: LockedAxes,
    #[bundle]
    transform: BndTransform,
    #[bundle]
    collider: BndCollider,
    #[bundle]
    sprite: BndSprite,
}

impl Bullet {
    pub fn new() -> Self {
        Self {
            gravity_scale: GravityScale(0.0),
            locked_axes: LockedAxes::ROTATION_LOCKED,
            transform: BndTransform::new(0.0, 1.0, 0.0),
            collider: BndCollider::new(Collider::ball(0.5)),
            sprite: BndSprite::new("bullet/texture.png", 0.5),
        }
    }
}
