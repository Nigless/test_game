use std::f32::consts;

use bevy::{
    core::Name,
    prelude::{Bundle, Projection},
    utils::default,
};
use bevy_rapier3d::prelude::{
    Collider, ColliderMassProperties, Friction, LockedAxes, RigidBody, Velocity,
};

use crate::shape_caster::ShapeCaster;

use super::{
    components::*, COLLIDER_CROUCHING_HALF_HEIGHT, COLLIDER_HALF_HEIGHT, COLLIDER_RADIUS,
    SKIN_WIDTH,
};

use bevy::prelude::*;

const CAST_DISTANCE: f32 =
    COLLIDER_HALF_HEIGHT - COLLIDER_CROUCHING_HALF_HEIGHT + COLLIDER_HALF_HEIGHT + SKIN_WIDTH;

#[derive(Bundle)]
pub struct GhostBundle {
    name: Name,
    parameters: Parameters,
    collider: Collider,
    body: RigidBody,
    velocity: Velocity,
    lock: LockedAxes,
    friction: Friction,
    mass: ColliderMassProperties,
}

impl GhostBundle {
    pub fn new() -> Self {
        Self {
            name: Name::new("ghost"),
            collider: Collider::capsule_y(COLLIDER_HALF_HEIGHT, COLLIDER_RADIUS),
            body: RigidBody::Dynamic,
            parameters: default(),
            velocity: Velocity::default(),
            lock: LockedAxes::ROTATION_LOCKED,
            friction: Friction::new(0.0),
            mass: ColliderMassProperties::Mass(65.0),
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
                Collider::ball(COLLIDER_RADIUS - SKIN_WIDTH),
                Vec3::Y * CAST_DISTANCE,
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
                Collider::ball(COLLIDER_RADIUS - SKIN_WIDTH),
                Vec3::NEG_Y * CAST_DISTANCE,
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