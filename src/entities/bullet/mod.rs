use super::utils::bnd_collider::BndCollider;
use super::utils::bnd_sprite::BndSprite;
use super::utils::bnd_transform::BndTransform;
use bevy_rapier3d::prelude::Collider;

use bevy::prelude::*;

#[derive(Bundle)]
pub struct Bullet {
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
            transform: BndTransform::new(1.0, 2.0, 0.0),
            collider: BndCollider::new(Collider::ball(0.1)),
            sprite: BndSprite::new("bullet/texture.png"),
        }
    }
}
