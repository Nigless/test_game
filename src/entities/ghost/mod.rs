use std::f32::consts;

use crate::camera_controller::CameraController;
use crate::control::{Control, ControlSystems, Input};
use crate::lib::move_toward;
use crate::linker::Linker;
use crate::shape_caster::{ShapeCaster, ShapeCasterSystems};

use bevy_rapier3d::dynamics::{GravityScale, Velocity};

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

pub struct GhostPlugin;

impl Plugin for GhostPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Parameters>()
            .register_type::<Status>()
            .add_systems(First, resolve)
            .add_systems(
                PreUpdate,
                (
                    ground_check.after(ShapeCasterSystems),
                    collider.after(ControlSystems).after(ShapeCasterSystems),
                    gravity,
                    looking.after(ControlSystems),
                    moving.after(ControlSystems).after(ground_check),
                    falling
                        .after(ControlSystems)
                        .after(ground_check)
                        .run_if(|input: Res<Input>| input.moving.length() > 0.0),
                    jumping
                        .after(ControlSystems)
                        .after(ground_check)
                        .run_if(|input: Res<Input>| input.jumping),
                ),
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
    mut entity_q: Query<(&Linker, &mut Status)>,
    caster_q: Query<&ShapeCaster, Without<Parameters>>,
    config: Res<RapierConfiguration>,
) {
    for (linker, mut status) in entity_q.iter_mut() {
        status.can_standup = true;

        status.surface = None;

        let cast_down = caster_q.get(*linker.get("cast_down").unwrap()).unwrap();

        if cast_down.result.is_none() {
            continue;
        }

        let cast_up = caster_q.get(*linker.get("cast_up").unwrap()).unwrap();

        let cast_down_result = cast_down.result.as_ref().unwrap();

        if let Some(cast_up_result) = &cast_up.result {
            if cast_up_result.distance + cast_down_result.distance
                < (COLLIDER_HALF_HEIGHT + SKIN_WIDTH) * 2.0
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

        if cast_down_result.distance > status.current_collider_height + SKIN_WIDTH + GROUND_WIDTH {
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
    config: Res<RapierConfiguration>,
    input: Res<Input>,
    time: Res<Time>,
    mut entity_q: Query<(&mut Collider, &mut Status, &mut Transform, &Linker), With<Control>>,
    caster_q: Query<&ShapeCaster, Without<Status>>,
    mut head_q: Query<&mut Transform, Without<Status>>,
) {
    for (mut collider, mut status, mut transform, linker) in entity_q.iter_mut() {
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

        let mut height_diff = status.current_collider_height.lerp(
            target_height,
            time.delta_seconds() * COLLIDER_TRANSITION_SPEED,
        ) - status.current_collider_height;

        if height_diff.abs() < 0.0001 {
            height_diff = target_height - status.current_collider_height;
        }

        let original_height = status.current_collider_height;

        status.current_collider_height += height_diff;

        head_transform.translation += Vec3::Y * height_diff;

        *collider = Collider::capsule_y(status.current_collider_height, COLLIDER_RADIUS);

        let cast_down = caster_q.get(*linker.get("cast_down").unwrap()).unwrap();

        if cast_down.result.is_none() {
            continue;
        }

        let result = cast_down.result.as_ref().unwrap();

        if height_diff < 0.0 {
            if result.distance < original_height + SKIN_WIDTH + GROUND_WIDTH {
                transform.translation += -config.gravity.normalize() * height_diff;
            }

            continue;
        }

        if result.distance > status.current_collider_height + SKIN_WIDTH + GROUND_WIDTH {
            continue;
        }

        transform.translation += -config.gravity.normalize()
            * (status.current_collider_height + SKIN_WIDTH - result.distance);
    }
}

fn gravity(
    mut entity_q: Query<(&mut Velocity, Option<&GravityScale>)>,
    time: Res<Time>,
    config: Res<RapierConfiguration>,
) {
    for (mut velocity, gravity) in entity_q.iter_mut() {
        let mut scale = 1.0;

        if let Some(gravity) = gravity {
            scale = gravity.0
        }

        velocity.linvel += config.gravity * scale * time.delta_seconds();
    }
}

fn moving(
    input: Res<Input>,
    mut entity_q: Query<(&mut Velocity, &Transform, &Parameters, &Status), With<Control>>,
) {
    for (mut velocity, transform, parameters, status) in entity_q.iter_mut() {
        if status.surface.is_none() {
            continue;
        }

        let mut speed = parameters.walking_speed;

        if input.running && input.moving.y <= 0.0 {
            speed = parameters.running_speed;
        }

        if input.crouching || !status.can_standup {
            speed = parameters.crouching_speed;
        }

        let ground_surface = status.surface.unwrap();

        let direction = Quat::from_rotation_arc(Vec3::Y, ground_surface)
            * transform.rotation
            * Vec3::new(input.moving.x, 0.0, input.moving.y).normalize_or_zero();

        let vertical_velocity = velocity.linvel.project_onto(ground_surface);

        let mut horizontal_velocity = velocity.linvel - vertical_velocity;

        horizontal_velocity = move_toward(
            horizontal_velocity,
            direction * speed,
            parameters.standing_acceleration,
        );

        velocity.linvel = horizontal_velocity + vertical_velocity;
    }
}

fn falling(
    config: Res<RapierConfiguration>,
    input: Res<Input>,
    mut entity_q: Query<(&mut Velocity, &Transform, &Parameters, &Status)>,
) {
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
            acceleration,
        );

        velocity.linvel = horizontal_velocity + vertical_velocity;
    }
}

fn jumping(
    input: Res<Input>,
    mut entity_q: Query<(&mut Velocity, &Parameters, &Status), With<Control>>,
) {
    for (mut velocity, parameters, status) in entity_q.iter_mut() {
        if status.surface.is_none() {
            continue;
        }

        let mut jump_high = parameters.standing_jump_height;

        if input.crouching || !status.can_standup {
            jump_high = parameters.crouching_jump_height;
        }

        let ground_surface = status.surface.unwrap();

        velocity.linvel = velocity.linvel - velocity.linvel.project_onto(ground_surface)
            + ground_surface * jump_high;
    }
}
