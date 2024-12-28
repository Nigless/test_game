use std::f32::consts;

use bevy::{
    core::Name,
    prelude::{Bundle, Projection},
    utils::default,
};
use bevy_rapier3d::prelude::Collider;

use crate::{character_body::CharacterBody, shape_caster::ShapeCaster};

use super::{
    components::*, COLLIDER_CROUCHING_HALF_HEIGHT, COLLIDER_HALF_HEIGHT, COLLIDER_RADIUS,
    SKIN_WIDTH,
};

use bevy::prelude::*;

#[derive(Bundle)]
pub struct GhostBundle {
    name: Name,
    parameters: Parameters,
    collider: Collider,
    character_body: CharacterBody,
}

impl GhostBundle {
    pub fn new() -> Self {
        Self {
            name: Name::new("ghost"),
            collider: Collider::capsule_y(COLLIDER_HALF_HEIGHT, COLLIDER_RADIUS),
            character_body: CharacterBody::default().skin_width(SKIN_WIDTH),
            parameters: default(),
        }
    }
}

#[derive(Bundle)]
pub struct GhostCamera {
    name: Name,
    camera: Camera3d,
    projection: Projection,
}

impl GhostCamera {
    pub fn new() -> Self {
        Self {
            name: Name::new("camera"),
            camera: Camera3d::default(),
            projection: Projection::Perspective(PerspectiveProjection {
                fov: consts::PI / 2.0,
                ..default()
            }),
        }
    }
}

#[derive(Bundle)]
pub struct GhostCastUp {
    name: Name,
    caster: ShapeCaster,
}

impl GhostCastUp {
    pub fn new() -> Self {
        Self {
            name: Name::new("cast_up"),
            caster: ShapeCaster::new(
                Collider::ball(COLLIDER_RADIUS),
                Vec3::Y
                    * (COLLIDER_HALF_HEIGHT - COLLIDER_CROUCHING_HALF_HEIGHT
                        + COLLIDER_HALF_HEIGHT),
            )
            .exclude_parent(),
        }
    }
}

#[derive(Bundle)]
pub struct GhostCastDown {
    name: Name,
    caster: ShapeCaster,
}

impl GhostCastDown {
    pub fn new() -> Self {
        Self {
            name: Name::new("cast_down"),
            caster: ShapeCaster::new(
                Collider::ball(COLLIDER_RADIUS),
                Vec3::NEG_Y
                    * (COLLIDER_HALF_HEIGHT - COLLIDER_CROUCHING_HALF_HEIGHT
                        + COLLIDER_HALF_HEIGHT),
            )
            .exclude_parent(),
        }
    }
}

#[derive(Bundle)]
pub struct GhostHead {
    name: Name,
    transform: Transform,
}

impl GhostHead {
    pub fn new(position: Vec3) -> Self {
        Self {
            name: Name::new("head"),
            transform: Transform {
                translation: position,
                ..default()
            },
        }
    }
}
