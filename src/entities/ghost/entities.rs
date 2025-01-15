use std::f32::consts;

use bevy::{
    audio::Volume,
    core::Name,
    prelude::{Bundle, Projection},
    utils::default,
};
use bevy_rapier3d::prelude::{
    CoefficientCombineRule, Collider, ColliderMassProperties, Friction, LockedAxes, RigidBody,
};

use crate::{liquid::VolumeScale, ray_caster::RayCaster, shape_caster::ShapeCaster};

use super::{
    components::*, COLLIDER_CROUCHING_HALF_HEIGHT, COLLIDER_HALF_HEIGHT, COLLIDER_RADIUS,
    HAND_DISTANCE, SKIN_WIDTH,
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
    lock: LockedAxes,
    friction: Friction,
    mass: ColliderMassProperties,
    volume_scale: VolumeScale,
}

impl GhostBundle {
    pub fn new() -> Self {
        Self {
            name: Name::new("ghost"),
            collider: Collider::capsule_y(COLLIDER_HALF_HEIGHT, COLLIDER_RADIUS),
            body: RigidBody::Dynamic,
            lock: LockedAxes::ROTATION_LOCKED,
            friction: Friction {
                coefficient: 0.0,
                combine_rule: CoefficientCombineRule::Multiply,
            },
            mass: ColliderMassProperties::Mass(65.0),
            parameters: default(),
            volume_scale: VolumeScale::new(0.5),
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
pub struct RayCast {
    name: Name,
    caster: RayCaster,
}

impl RayCast {
    pub fn new(exclude: Entity) -> Self {
        Self {
            name: Name::new("ray_cast"),
            caster: RayCaster::new(Vec3::NEG_Z * HAND_DISTANCE).exclude(exclude),
        }
    }
}

#[derive(Bundle)]
pub struct ShapeCast {
    name: Name,
    caster: ShapeCaster,
}

impl ShapeCast {
    pub fn up(exclude: Entity) -> Self {
        Self {
            name: Name::new("cast_up"),
            caster: ShapeCaster::new(
                Collider::ball(COLLIDER_RADIUS - SKIN_WIDTH),
                Vec3::Y * CAST_DISTANCE,
            )
            .fixed_update()
            .exclude(exclude),
        }
    }

    pub fn down(exclude: Entity) -> Self {
        Self {
            name: Name::new("cast_down"),
            caster: ShapeCaster::new(
                Collider::ball(COLLIDER_RADIUS - SKIN_WIDTH),
                Vec3::NEG_Y * CAST_DISTANCE,
            )
            .fixed_update()
            .exclude(exclude),
        }
    }
}

#[derive(Bundle)]
pub struct Head {
    name: Name,
    transform: Transform,
}

impl Head {
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
