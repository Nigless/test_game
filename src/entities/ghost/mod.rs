use std::f32::consts;

use crate::camera_controller::CameraController;
use crate::character_body::{CharacterBody, CharacterBodySystems};
use crate::control::{Control, Input};
use crate::lib::move_toward;
use crate::linker::Linker;
use crate::shape_caster::{ShapeCaster, ShapeCasterSystems};

use bevy_rapier3d::dynamics::{GravityScale, Velocity};

use bevy_rapier3d::parry::math::{Point, Vector};
use bevy_rapier3d::parry::shape::SharedShape;
use bevy_rapier3d::plugin::RapierConfiguration;
use bevy_rapier3d::prelude::Collider;

use bevy::prelude::*;
use components::{Parameters, Status, Unresolved};
use entities::{GhostCamera, GhostCastDown, GhostCastUp, GhostHead};
mod components;
mod entities;
pub use entities::GhostBundle;

pub const MAX_SLOPE_ANGLE: f32 = consts::PI / 3.8;
pub const COLLIDER_TRANSITION_SPEED: f32 = 20.0;
pub const COLLIDER_RADIUS: f32 = 0.3;
pub const GROUND_WIDTH: f32 = 0.01;
pub const SKIN_WIDTH: f32 = 0.01;
pub const COLLIDER_HALF_HEIGHT: f32 = 1.0 - COLLIDER_RADIUS;
pub const COLLIDER_CROUCHING_HALF_HEIGHT: f32 = COLLIDER_HALF_HEIGHT * 0.4;

#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub enum GhostSystems {
    Resolve,
    Prepare,
    Update,
}

pub struct GhostPlugin;

impl Plugin for GhostPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Parameters>()
            .register_type::<Status>()
            .configure_sets(
                PreUpdate,
                (GhostSystems::Prepare, GhostSystems::Update)
                    .chain()
                    .after(ShapeCasterSystems)
                    .before(CharacterBodySystems),
            )
            .add_systems(First, resolve.in_set(GhostSystems::Resolve))
            .add_systems(
                PreUpdate,
                (ground_check, looking).in_set(GhostSystems::Prepare),
            )
            .add_systems(
                PreUpdate,
                (
                    collider,
                    gravity,
                    moving,
                    falling.run_if(|input: Res<Input>| input.moving.length() > 0.0),
                    jumping.run_if(|input: Res<Input>| input.jumping),
                )
                    .in_set(GhostSystems::Update),
            );
    }
}

fn resolve(mut commands: Commands, mut entity_q: Query<Entity, With<Unresolved>>) {
    for entity in entity_q.iter_mut() {
        let camera = commands.spawn(GhostCamera::new()).id();

        let head = commands
            .spawn(GhostHead::new(Vec3::new(0.0, COLLIDER_HALF_HEIGHT, 0.0)))
            .add_child(camera)
            .id();

        let cast_up = commands.spawn(GhostCastUp::new()).id();

        let cast_down = commands.spawn(GhostCastDown::new()).id();

        commands
            .entity(entity)
            .remove::<Unresolved>()
            .insert((
                CameraController::new(camera),
                Linker::new()
                    .with_link("head", head)
                    .with_link("cast_up", cast_up)
                    .with_link("cast_down", cast_down),
            ))
            .add_child(head)
            .add_child(cast_up)
            .add_child(cast_down);
    }
}

fn ground_check(
    mut entity_q: Query<(&Linker, &mut Status, &CharacterBody)>,
    caster_q: Query<&ShapeCaster, Without<Status>>,
    config_q: Query<&RapierConfiguration, Without<ShapeCaster>>,
) {
    let config = config_q.get_single().unwrap();

    for (linker, mut status, body) in entity_q.iter_mut() {
        status.can_standup = true;

        status.surface = None;

        let cast_down = caster_q.get(*linker.get("cast_down").unwrap()).unwrap();

        if cast_down.result.is_none() {
            continue;
        }

        let Some(cast_down_result) = cast_down.result.as_ref() else {
            continue;
        };

        let cast_up = caster_q.get(*linker.get("cast_up").unwrap()).unwrap();

        if let Some(cast_up_result) = &cast_up.result {
            if cast_up_result.distance + cast_down_result.distance
                < (COLLIDER_HALF_HEIGHT + body.skin_width) * 2.0
            {
                status.can_standup = false;
            }
        }

        if cast_down_result
            .normal
            .angle_between(-config.gravity.normalize())
            > MAX_SLOPE_ANGLE
        {
            continue;
        }

        if cast_down_result.distance
            > status.current_collider_height + body.skin_width + GROUND_WIDTH
        {
            continue;
        }

        status.surface = Some(cast_down_result.normal);
    }
}

fn looking(
    input: Res<Input>,
    mut entity_q: Query<(&mut Transform, &Linker), (With<Control>, With<Parameters>)>,
    mut head_q: Query<&mut Transform, Without<Parameters>>,
) {
    for (mut transform, linker) in entity_q.iter_mut() {
        let mut head_transform = head_q.get_mut(*linker.get("head").unwrap()).unwrap();

        let rotation = head_transform.rotation * Quat::from_rotation_x(input.looking_around.y);

        if (rotation * Vec3::Y).y >= 0.0 {
            head_transform.rotation = rotation;
        }

        transform.rotation *= Quat::from_rotation_y(input.looking_around.x);
    }
}

fn collider(
    input: Res<Input>,
    time: Res<Time>,
    mut entity_q: Query<
        (
            &mut Collider,
            &mut Status,
            &mut Transform,
            &Linker,
            &CharacterBody,
        ),
        With<Control>,
    >,
    caster_q: Query<&ShapeCaster, Without<Status>>,
    mut head_q: Query<&mut Transform, Without<Status>>,
    config_q: Query<&RapierConfiguration, Without<ShapeCaster>>,
) {
    let config = config_q.get_single().unwrap();

    for (mut collider, mut status, mut transform, linker, body) in entity_q.iter_mut() {
        let target_height = if input.crouching {
            COLLIDER_CROUCHING_HALF_HEIGHT
        } else if status.can_standup {
            COLLIDER_HALF_HEIGHT
        } else {
            continue;
        };

        if status.current_collider_height == target_height {
            continue;
        }

        let mut head_transform = head_q.get_mut(*linker.get("head").unwrap()).unwrap();

        let mut height_diff = status
            .current_collider_height
            .lerp(target_height, time.delta_secs() * COLLIDER_TRANSITION_SPEED)
            - status.current_collider_height;

        if height_diff.abs() < 0.0001
            || height_diff.abs() > (target_height - status.current_collider_height).abs()
        {
            height_diff = target_height - status.current_collider_height;
        }

        let original_height = status.current_collider_height;

        status.current_collider_height += height_diff;

        head_transform.translation += Vec3::Y * height_diff;

        *collider = Collider::capsule_y(status.current_collider_height, COLLIDER_RADIUS);

        let cast_down = caster_q.get(*linker.get("cast_down").unwrap()).unwrap();
        let cast_up = caster_q.get(*linker.get("cast_up").unwrap()).unwrap();

        if let Some(result) = cast_down.result.as_ref() {
            if height_diff < 0.0 {
                if result.distance < original_height + body.skin_width + GROUND_WIDTH {
                    transform.translation += -config.gravity.normalize() * height_diff;
                }

                continue;
            }

            if result.distance < status.current_collider_height + body.skin_width + GROUND_WIDTH {
                transform.translation += -config.gravity.normalize()
                    * (status.current_collider_height + body.skin_width - result.distance);

                continue;
            }
        }

        if let Some(result) = cast_up.result.as_ref() {
            if result.distance > status.current_collider_height + body.skin_width + GROUND_WIDTH {
                continue;
            }

            transform.translation -= -config.gravity.normalize()
                * (status.current_collider_height + body.skin_width - result.distance);
        };
    }
}

fn gravity(
    mut entity_q: Query<(&mut Velocity, Option<&GravityScale>)>,
    time: Res<Time>,
    config_q: Query<&RapierConfiguration, Without<Velocity>>,
) {
    let config = config_q.get_single().unwrap();

    for (mut velocity, gravity) in entity_q.iter_mut() {
        let mut scale = 1.0;

        if let Some(gravity) = gravity {
            scale = gravity.0
        }

        velocity.linvel += config.gravity * scale * time.delta_secs();
    }
}

fn moving(
    time: Res<Time>,
    input: Res<Input>,
    mut entity_q: Query<(
        &mut Velocity,
        &Transform,
        &Parameters,
        &Status,
        Option<&Control>,
    )>,
) {
    for (mut velocity, transform, parameters, status, control) in entity_q.iter_mut() {
        let Some(ground_surface) = status.surface else {
            continue;
        };
        let mut speed = parameters.walking_speed;

        let mut direction = Vec3::ZERO;

        if control.is_some() {
            if input.running && input.moving.y <= 0.0 {
                speed = parameters.running_speed;
            }

            if input.crouching || !status.can_standup {
                speed = parameters.crouching_speed;
            }

            direction = Quat::from_rotation_arc(Vec3::Y, ground_surface)
                * transform.rotation
                * Vec3::new(input.moving.x, 0.0, input.moving.y).normalize_or_zero();
        };

        let vertical_velocity = velocity.linvel.project_onto(ground_surface);

        let mut horizontal_velocity = velocity.linvel - vertical_velocity;

        horizontal_velocity = move_toward(
            horizontal_velocity,
            direction * speed,
            parameters.standing_acceleration * time.delta_secs(),
        );

        velocity.linvel = horizontal_velocity + vertical_velocity;
    }
}

fn falling(
    time: Res<Time>,
    input: Res<Input>,
    mut entity_q: Query<(&mut Velocity, &Transform, &Parameters, &Status), With<Control>>,
    config_q: Query<&RapierConfiguration, Without<Status>>,
) {
    let config = config_q.get_single().unwrap();

    for (mut velocity, transform, parameters, status) in entity_q.iter_mut() {
        if status.surface.is_some() {
            continue;
        }

        let direction =
            transform.rotation * Vec3::new(input.moving.x, 0.0, input.moving.y).normalize_or_zero();

        let vertical_velocity = velocity.linvel.project_onto(-config.gravity.normalize());

        let mut horizontal_velocity = velocity.linvel - vertical_velocity;

        let acceleration = (((direction * parameters.falling_speed) - horizontal_velocity)
            .normalize_or_zero()
            .dot(direction)
            + 1.0)
            / 2.0
            * parameters.falling_acceleration;

        horizontal_velocity = move_toward(
            horizontal_velocity,
            direction * parameters.falling_speed,
            acceleration * time.delta_secs(),
        );

        velocity.linvel = horizontal_velocity + vertical_velocity;
    }
}

fn jumping(
    input: Res<Input>,
    mut entity_q: Query<(&mut Velocity, &Parameters, &Status), With<Control>>,
) {
    for (mut velocity, parameters, status) in entity_q.iter_mut() {
        let Some(ground_surface) = status.surface else {
            continue;
        };

        let mut jump_high = parameters.standing_jump_height;

        if input.crouching || !status.can_standup {
            jump_high = parameters.crouching_jump_height;
        }

        velocity.linvel = velocity.linvel - velocity.linvel.project_onto(ground_surface)
            + ground_surface * jump_high;
    }
}
