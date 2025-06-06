use std::{any::TypeId, f32::consts};

use bevy::{
    audio::Volume,
    core::Name,
    ecs::{component::ComponentId, world::DeferredWorld},
    image,
    prelude::{Bundle, Projection},
    state::commands,
    text::cosmic_text::ttf_parser::name,
    utils::default,
};
use bevy_rapier3d::prelude::{
    CoefficientCombineRule, Collider, ColliderMassProperties, Friction, GravityScale, LockedAxes,
    RigidBody, Velocity,
};

use crate::{
    plugins::{
        health::{DeadEvent, Health},
        input::Control,
        linker::Linker,
        liquid::VolumeScale,
        ray_caster::RayCaster,
        serializable::Serializable,
        shape_caster::ShapeCaster,
    },
    stores::game::PlayerDeadEvent,
};

use super::{
    components::{self, *},
    COLLIDER_CROUCHING_HALF_HEIGHT, COLLIDER_HALF_HEIGHT, COLLIDER_RADIUS, HAND_DISTANCE,
    SKIN_WIDTH,
};

use bevy::prelude::*;

const CAST_DISTANCE: f32 =
    COLLIDER_HALF_HEIGHT - COLLIDER_CROUCHING_HALF_HEIGHT + COLLIDER_HALF_HEIGHT + SKIN_WIDTH;

#[derive(Component, Reflect, PartialEq, Clone)]
#[reflect(Component)]
#[component(on_add = spawn)]
#[require(GravityScale, components::State, Velocity, Transform)]
pub struct Player {
    pub walking_speed: f32,
    pub falling_speed: f32,
    pub running_speed: f32,
    pub crouching_speed: f32,
    pub swimming_speed: f32,
    pub falling_acceleration: f32,
    pub standing_acceleration: f32,
    pub standing_jump_height: f32,
    pub crouching_jump_height: f32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            walking_speed: 4.0,
            running_speed: 8.0,
            falling_speed: 2.0,
            falling_acceleration: 3.0,
            standing_acceleration: 40.0,
            standing_jump_height: 4.0,
            crouching_jump_height: 2.0,
            crouching_speed: 2.0,
            swimming_speed: 6.0,
        }
    }
}

fn spawn(mut world: DeferredWorld<'_>, entity: Entity, _: ComponentId) {
    let commands = &mut world.commands();

    commands
        .entity(entity)
        .insert((
            Serializable::default()
                .with::<Player>()
                .with::<components::State>()
                .with::<Transform>()
                .with::<Velocity>()
                .with::<Health>()
                .with::<Control>(),
            Health::new(100),
            Name::new("player"),
            Collider::capsule_y(COLLIDER_HALF_HEIGHT, COLLIDER_RADIUS),
            RigidBody::Dynamic,
            LockedAxes::ROTATION_LOCKED,
            Friction {
                coefficient: 0.0,
                combine_rule: CoefficientCombineRule::Multiply,
            },
            ColliderMassProperties::Mass(65.0),
            VolumeScale::new(0.15),
        ))
        .observe(handle_death);

    let head = commands
        .spawn(Head::new(Vec3::Y * COLLIDER_HALF_HEIGHT).bundle())
        .set_parent(entity)
        .id();

    commands.spawn(PlayerCamera::bundle()).set_parent(head);

    let ray_cast = commands
        .spawn(RayCast::new(entity).bundle())
        .set_parent(head)
        .id();

    let cast_up = commands
        .spawn(ShapeCast::up(entity).bundle())
        .set_parent(entity)
        .id();

    let cast_down = commands
        .spawn(ShapeCast::down(entity).bundle())
        .set_parent(entity)
        .id();

    commands.entity(entity).insert((Linker::new()
        .with_link("head", head)
        .with_link("ray_cast", ray_cast)
        .with_link("cast_up", cast_up)
        .with_link("cast_down", cast_down),));
}

fn handle_death(s: Trigger<DeadEvent>, mut commands: Commands) {
    println!("{}", s.entity());

    commands.trigger(PlayerDeadEvent);
}

pub struct PlayerCamera;

impl PlayerCamera {
    pub fn bundle() -> impl Bundle {
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

pub struct RayCast {
    exclude: Entity,
}

impl RayCast {
    fn new(exclude: Entity) -> Self {
        Self { exclude }
    }

    fn bundle(self) -> impl Bundle {
        (
            Name::new("ray_cast"),
            RayCaster::new(Vec3::NEG_Z * HAND_DISTANCE).exclude(self.exclude),
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

    pub fn bundle(self) -> impl Bundle {
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

pub struct Head {
    position: Vec3,
}

impl Head {
    fn new(position: Vec3) -> Self {
        Self { position }
    }

    pub fn bundle(self) -> impl Bundle {
        (
            Name::new("head"),
            Transform {
                translation: self.position,
                ..default()
            },
        )
    }
}
