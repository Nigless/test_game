use std::f32::consts;

use bevy::{
    core::Name,
    prelude::{Bundle, Camera3dBundle, Projection, TransformBundle},
    utils::default,
};
use bevy_rapier3d::prelude::{Collider, CollisionGroups, GravityScale, Group, Sensor, Velocity};

use crate::{character_body::CharacterBody, linker::Linker, shape_caster::ShapeCaster};

use super::{
    components::*, COLLIDER_CROUCHING_HALF_HEIGHT, COLLIDER_HALF_HEIGHT, COLLIDER_RADIUS,
    SKIN_WIDTH,
};

use bevy::prelude::*;

#[derive(Bundle)]
pub struct GhostBundle {
    unresolved: Unresolved,
    name: Name,
    parameters: Parameters,
    transform: TransformBundle,
    velocity: Velocity,
    gravity: GravityScale,
    collider: Collider,
    sensor: Sensor,
    body: CharacterBody,
    status: Status,
}

impl GhostBundle {
    pub fn new() -> Self {
        Self {
            unresolved: Unresolved,
            name: Name::new("ghost"),
            collider: Collider::capsule_y(COLLIDER_HALF_HEIGHT, COLLIDER_RADIUS),
            body: CharacterBody::default().skin_width(SKIN_WIDTH),
            sensor: default(),
            parameters: default(),
            transform: default(),
            velocity: default(),
            gravity: default(),
            status: default(),
        }
    }
}

#[derive(Bundle)]
pub struct GhostCamera {
    name: Name,
    camera: Camera3dBundle,
}

impl GhostCamera {
    pub fn new() -> Self {
        Self {
            name: Name::new("camera"),
            camera: Camera3dBundle {
                projection: Projection::Perspective(PerspectiveProjection {
                    fov: consts::PI / 2.0,
                    ..default()
                }),
                ..default()
            },
        }
    }
}

#[derive(Bundle)]
pub struct GhostCastUp {
    name: Name,
    caster: ShapeCaster,
    collider: Collider,
    transform: TransformBundle,
    sensor: Sensor,
}

impl GhostCastUp {
    pub fn new() -> Self {
        Self {
            name: Name::new("cast_up"),
            caster: ShapeCaster::new(
                Vec3::Y
                    * (COLLIDER_HALF_HEIGHT - COLLIDER_CROUCHING_HALF_HEIGHT
                        + COLLIDER_HALF_HEIGHT),
            ),
            collider: Collider::ball(COLLIDER_RADIUS),
            sensor: default(),
            transform: default(),
        }
    }
}

#[derive(Bundle)]
pub struct GhostCastDown {
    name: Name,
    caster: ShapeCaster,
    collider: Collider,
    transform: TransformBundle,
    sensor: Sensor,
}

impl GhostCastDown {
    pub fn new() -> Self {
        Self {
            name: Name::new("cast_down"),
            caster: ShapeCaster::new(
                Vec3::NEG_Y
                    * (COLLIDER_HALF_HEIGHT - COLLIDER_CROUCHING_HALF_HEIGHT
                        + COLLIDER_HALF_HEIGHT),
            ),
            collider: Collider::ball(COLLIDER_RADIUS),
            sensor: default(),
            transform: default(),
        }
    }
}

#[derive(Bundle)]
pub struct GhostHead {
    name: Name,
    transform: Transform,
    global_transform: GlobalTransform,
}

impl GhostHead {
    pub fn new(position: Vec3) -> Self {
        Self {
            name: Name::new("head"),
            transform: Transform {
                translation: position,
                ..default()
            },
            global_transform: GlobalTransform::default(),
        }
    }
}
