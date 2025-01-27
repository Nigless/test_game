use std::f32::consts;

use bevy::{
    audio::Volume,
    core::Name,
    prelude::{Bundle, Projection},
    text::cosmic_text::ttf_parser::name,
    utils::default,
};
use bevy_rapier3d::prelude::{
    CoefficientCombineRule, Collider, ColliderMassProperties, Friction, LockedAxes, RigidBody,
};

use crate::{
    camera_controller::CameraController, library::Spawnable, linker::Linker, liquid::VolumeScale,
    ray_caster::RayCaster, shape_caster::ShapeCaster,
};

use super::{
    components::*, COLLIDER_CROUCHING_HALF_HEIGHT, COLLIDER_HALF_HEIGHT, COLLIDER_RADIUS,
    HAND_DISTANCE, SKIN_WIDTH,
};

use bevy::prelude::*;

const CAST_DISTANCE: f32 =
    COLLIDER_HALF_HEIGHT - COLLIDER_CROUCHING_HALF_HEIGHT + COLLIDER_HALF_HEIGHT + SKIN_WIDTH;

pub struct Player;

impl Player {
    fn bundle() -> impl Bundle {
        (
            Name::new("player"),
            Collider::capsule_y(COLLIDER_HALF_HEIGHT, COLLIDER_RADIUS),
            RigidBody::Dynamic,
            LockedAxes::ROTATION_LOCKED,
            Friction {
                coefficient: 0.0,
                combine_rule: CoefficientCombineRule::Multiply,
            },
            ColliderMassProperties::Mass(65.0),
            Parameters::default(),
            VolumeScale::new(0.2),
        )
    }
}

impl Spawnable for Player {
    fn spawn(&self, commands: &mut Commands) -> Entity {
        let player = commands.spawn(Self::bundle()).id();

        let camera = commands.spawn(GhostCamera::bundle()).id();

        let ray_cast = commands.spawn(RayCast::bundle(player)).id();

        let head = commands
            .spawn(Head::bundle(Vec3::Y * COLLIDER_HALF_HEIGHT))
            .add_child(camera)
            .add_child(ray_cast)
            .id();

        let cast_up = commands.spawn(ShapeCast::up(player).bundle()).id();
        let cast_down = commands.spawn(ShapeCast::down(player).bundle()).id();

        commands
            .entity(player)
            .insert((
                CameraController::new(camera),
                Linker::new()
                    .with_link("head", head)
                    .with_link("ray_cast", ray_cast)
                    .with_link("cast_up", cast_up)
                    .with_link("cast_down", cast_down),
            ))
            .add_child(head)
            .add_child(cast_up)
            .add_child(cast_down)
            .id()
    }
}

pub struct GhostCamera;

impl GhostCamera {
    fn bundle() -> impl Bundle {
        (
            Name::new("camera"),
            Camera3d::default(),
            Projection::Perspective(PerspectiveProjection {
                fov: consts::PI / 2.0,
                ..default()
            }),
        )
    }
}

pub struct RayCast;

impl RayCast {
    pub fn bundle(exclude: Entity) -> impl Bundle {
        (
            Name::new("ray_cast"),
            RayCaster::new(Vec3::NEG_Z * HAND_DISTANCE).exclude(exclude),
        )
    }
}

pub struct ShapeCast<'s> {
    exclude: Entity,
    name: &'s str,
    direction: Vec3,
}

impl<'s> ShapeCast<'s> {
    pub fn up(exclude: Entity) -> Self {
        Self {
            exclude,
            name: "cast_up",
            direction: Vec3::Y,
        }
    }

    pub fn down(exclude: Entity) -> Self {
        Self {
            exclude,
            name: "cast_down",
            direction: Vec3::NEG_Y,
        }
    }

    fn bundle(&self) -> impl Bundle {
        (
            Name::new(self.name.to_owned()),
            ShapeCaster::new(
                Collider::ball(COLLIDER_RADIUS - SKIN_WIDTH),
                self.direction * CAST_DISTANCE,
            )
            .fixed_update()
            .exclude(self.exclude),
        )
    }
}

pub struct Head;

impl Head {
    pub fn bundle(position: Vec3) -> impl Bundle {
        (
            Name::new("head"),
            Transform {
                translation: position,
                ..default()
            },
        )
    }
}
