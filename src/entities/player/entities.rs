use std::{any::TypeId, f32::consts};

use bevy::{
    audio::Volume,
    core::Name,
    ecs::{component::ComponentId, world::DeferredWorld},
    prelude::{Bundle, Projection},
    text::cosmic_text::ttf_parser::name,
    utils::default,
};
use bevy_rapier3d::prelude::{
    CoefficientCombineRule, Collider, ColliderMassProperties, Friction, GravityScale, LockedAxes,
    RigidBody, Velocity,
};

use crate::{
    control::Control, library::Spawnable, linker::Linker, liquid::VolumeScale,
    ray_caster::RayCaster, saves::Serializable, shape_caster::ShapeCaster,
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

    commands.entity(entity).insert((
        Serializable::default()
            .with::<Player>()
            .with::<components::State>()
            .with::<Transform>()
            .with::<Velocity>()
            .with::<Control>(),
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
    ));

    let head = Head::new(Vec3::Y * COLLIDER_HALF_HEIGHT)
        .spawn(commands)
        .set_parent(entity)
        .id();

    PlayerCamera.spawn(commands).set_parent(head);

    let ray_cast = RayCast::new(entity).spawn(commands).set_parent(head).id();

    let cast_up = ShapeCast::up(entity)
        .spawn(commands)
        .set_parent(entity)
        .id();

    let cast_down = ShapeCast::down(entity)
        .spawn(commands)
        .set_parent(entity)
        .id();

    commands.entity(entity).insert((Linker::new()
        .with_link("head", head)
        .with_link("ray_cast", ray_cast)
        .with_link("cast_up", cast_up)
        .with_link("cast_down", cast_down),));
}

impl Spawnable for Player {
    fn spawn<'a>(&self, commands: &'a mut Commands) -> EntityCommands<'a> {
        commands.spawn(self.clone())
    }
}

pub struct PlayerCamera;

impl Spawnable for PlayerCamera {
    fn spawn<'a>(&self, commands: &'a mut Commands) -> EntityCommands<'a> {
        commands.spawn((
            Name::new("camera"),
            Camera3d::default(),
            Projection::Perspective(PerspectiveProjection {
                fov: consts::PI / 2.0,
                ..default()
            }),
        ))
    }
}

pub struct RayCast {
    exclude: Entity,
}

impl RayCast {
    fn new(exclude: Entity) -> Self {
        Self { exclude }
    }
}

impl Spawnable for RayCast {
    fn spawn<'a>(&self, commands: &'a mut Commands) -> EntityCommands<'a> {
        commands.spawn((
            Name::new("ray_cast"),
            RayCaster::new(Vec3::NEG_Z * HAND_DISTANCE).exclude(self.exclude),
        ))
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
}

impl<'s> Spawnable for ShapeCast<'s> {
    fn spawn<'a>(&self, commands: &'a mut Commands) -> EntityCommands<'a> {
        commands.spawn((
            Name::new(self.name.to_owned()),
            ShapeCaster::new(
                Collider::ball(COLLIDER_RADIUS - SKIN_WIDTH),
                self.direction * CAST_DISTANCE,
            )
            .fixed_update()
            .exclude(self.exclude),
        ))
    }
}

pub struct Head {
    position: Vec3,
}

impl Head {
    fn new(position: Vec3) -> Self {
        Self { position }
    }
}

impl Spawnable for Head {
    fn spawn<'a>(&self, commands: &'a mut Commands) -> EntityCommands<'a> {
        commands.spawn((
            Name::new("head"),
            Transform {
                translation: self.position,
                ..default()
            },
        ))
    }
}
