use std::f32::consts;
use std::time::Duration;

use crate::camera_controller::CameraController;
use crate::control::{Control, ControlSystems, Input};
use crate::despawn::Despawn;
use crate::library::{move_toward, Spawnable};
use crate::linker::Linker;
use crate::liquid::{Floating, LiquidSystems};
use crate::ray_caster::RayCasterSystems;
use crate::shape_caster::{ShapeCaster, ShapeCasterSystems};

use bevy::animation::{animated_field, AnimationTargetId};
use bevy_rapier3d::dynamics::Velocity;

use bevy_rapier3d::plugin::{RapierConfiguration, RapierContext};
use bevy_rapier3d::prelude::{Collider, GravityScale, QueryFilter};

use bevy::prelude::*;
use components::{Parameters, Status};
use entities::{Head, PlayerCamera, RayCast, ShapeCast};
mod components;
mod entities;
pub use entities::Player;

use super::fireball::Fireball;

const MAX_SLOPE_ANGLE: f32 = consts::PI / 3.8;
const HAND_DISTANCE: f32 = 2.0;
const COLLIDER_TRANSITION_SPEED: f32 = 0.1;
const COLLIDER_RADIUS: f32 = 0.3;
const MAX_SURFACE_GAP: f32 = 0.03 + SKIN_WIDTH;
const SKIN_WIDTH: f32 = 0.05;
const COLLIDER_HALF_HEIGHT: f32 = 1.0 - COLLIDER_RADIUS;
const COLLIDER_CROUCHING_HALF_HEIGHT: f32 = COLLIDER_HALF_HEIGHT * 0.4;

#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
enum PlayerSystems {
    Update,
    Prepare,
    FixedUpdate,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Parameters>()
            .register_type::<Status>()
            .add_systems(
                PreUpdate,
                (camera, fireball)
                    .in_set(PlayerSystems::Update)
                    .after(ControlSystems)
                    .before(RayCasterSystems),
            )
            .configure_sets(
                FixedPreUpdate,
                (PlayerSystems::Prepare, PlayerSystems::FixedUpdate)
                    .chain()
                    .after(ShapeCasterSystems)
                    .after(LiquidSystems::Update),
            )
            .add_systems(
                FixedPreUpdate,
                (
                    ground_check.in_set(PlayerSystems::Prepare),
                    (
                        collider,
                        moving,
                        falling.run_if(|input: Res<Input>| input.moving.length() > 0.0),
                        swimming,
                        jumping,
                    )
                        .in_set(PlayerSystems::FixedUpdate),
                ),
            );
    }
}

fn fireball(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    entity_q: Query<&Linker, With<Status>>,
    camera_q: Query<&GlobalTransform>,
) {
    if !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }

    for linker in entity_q.iter() {
        let transform = camera_q.get(*linker.get("head").unwrap()).unwrap();

        let direction = transform.rotation() * Vec3::NEG_Z;

        let position = transform.translation() + direction;

        Fireball.spawn(&mut commands).insert((
            Transform::from_translation(position),
            Velocity::linear(direction * 50.0),
            Despawn::after(Duration::from_secs(10)),
        ));
    }
}

fn ground_check(
    mut entity_q: Query<(
        &Linker,
        &mut Status,
        &mut GravityScale,
        &mut Transform,
        &mut Velocity,
        Option<&Floating>,
    )>,
    time: Res<Time<Fixed>>,
    caster_q: Query<&ShapeCaster, Without<Status>>,
    config_q: Query<&RapierConfiguration, Without<ShapeCaster>>,
) {
    let config = config_q.get_single().unwrap();

    for (linker, mut status, mut gravity, mut transform, mut velocity, floating) in
        entity_q.iter_mut()
    {
        status.can_standup = true;

        status.surface = None;

        gravity.0 = 1.0;

        let cast_down = caster_q.get(*linker.get("cast_down").unwrap()).unwrap();

        let Some(cast_down_result) = cast_down.result.as_ref() else {
            continue;
        };

        let normal = cast_down_result.normal;

        let cast_up = caster_q.get(*linker.get("cast_up").unwrap()).unwrap();

        if let Some(cast_up_result) = &cast_up.result {
            if cast_up_result.distance + cast_down_result.distance < COLLIDER_HALF_HEIGHT * 2.0 {
                status.can_standup = false;
            }
        }

        if floating.is_some() {
            continue;
        }

        let gravity_direction = config.gravity.normalize();

        if normal.angle_between(-gravity_direction) > MAX_SLOPE_ANGLE {
            continue;
        }

        let ground_gap = cast_down_result.distance - SKIN_WIDTH - status.current_collider_height;

        if ground_gap > MAX_SURFACE_GAP {
            continue;
        }

        if ground_gap < SKIN_WIDTH {
            gravity.0 = 0.0;

            velocity.linvel = velocity.linvel.reject_from(normal);

            transform.translation += normal * (SKIN_WIDTH - ground_gap) * (time.delta_secs() / 0.5)
        }

        status.surface = Some(normal);
    }
}

fn camera(
    mut input: ResMut<Input>,
    mut entity_q: Query<(&mut Transform, &Linker), (With<Control>, With<Parameters>)>,
    mut head_q: Query<&mut Transform, Without<Parameters>>,
) {
    let looking = input.looking();

    for (mut transform, linker) in entity_q.iter_mut() {
        let mut head_transform = head_q.get_mut(*linker.get("head").unwrap()).unwrap();

        let rotation = head_transform.rotation * Quat::from_rotation_x(looking.y);

        if (rotation * Vec3::Y).y >= 0.0 {
            head_transform.rotation = rotation;
        }

        transform.rotation *= Quat::from_rotation_y(looking.x);
    }
}

fn collider(
    input: Res<Input>,
    time: Res<Time<Fixed>>,
    mut entity_q: Query<
        (
            &mut Collider,
            &mut Status,
            &mut Transform,
            &Linker,
            Option<&Floating>,
        ),
        With<Control>,
    >,
    caster_q: Query<&ShapeCaster, Without<Status>>,
    mut head_q: Query<&mut Transform, Without<Status>>,
    config_q: Query<&RapierConfiguration, Without<ShapeCaster>>,
) {
    let config = config_q.get_single().unwrap();

    for (mut collider, mut status, mut transform, linker, floating) in entity_q.iter_mut() {
        let cast_down = caster_q.get(*linker.get("cast_down").unwrap()).unwrap();
        let cast_up = caster_q.get(*linker.get("cast_up").unwrap()).unwrap();

        let distance_to_ground = |height: f32| {
            let Some(result) = cast_down.result.as_ref() else {
                return f32::INFINITY;
            };

            return result.distance - SKIN_WIDTH * 2.0 - height;
        };

        let distance_to_celling = |height: f32| {
            let Some(result) = cast_up.result.as_ref() else {
                return f32::INFINITY;
            };

            return result.distance - SKIN_WIDTH * 2.0 - height;
        };

        let get_height_diff = || {
            let mut target_height = if input.crouching {
                COLLIDER_CROUCHING_HALF_HEIGHT
            } else if status.can_standup {
                COLLIDER_HALF_HEIGHT
            } else {
                return 0.0;
            };

            if floating.is_some() && status.can_standup {
                target_height = COLLIDER_HALF_HEIGHT
            }

            if status.current_collider_height == target_height {
                return 0.0;
            }

            let delta = time.delta_secs() / COLLIDER_TRANSITION_SPEED;

            if delta > 1.0 {
                return target_height - status.current_collider_height;
            }

            if (target_height - status.current_collider_height).abs() < 0.0001 {
                return target_height - status.current_collider_height;
            }

            return status.current_collider_height.lerp(target_height, delta)
                - status.current_collider_height;
        };

        let height_diff = get_height_diff();

        let original_height = status.current_collider_height;

        let gravity_direction = config.gravity.normalize();

        if height_diff != 0.0 {
            let mut head_transform = head_q.get_mut(*linker.get("head").unwrap()).unwrap();

            status.current_collider_height += height_diff;

            head_transform.translation += Vec3::Y * height_diff;

            *collider = Collider::capsule_y(status.current_collider_height, COLLIDER_RADIUS);
        }

        // sitting down
        if status.current_collider_height < original_height {
            let distance = distance_to_ground(original_height);
            if distance <= 0.0 {
                transform.translation +=
                    gravity_direction * distance_to_ground(status.current_collider_height);
            }
            continue;
        }

        // standing up
        if status.current_collider_height > original_height {
            let distance = distance_to_ground(status.current_collider_height);

            if distance < 0.0 {
                transform.translation += gravity_direction * distance;

                continue;
            }

            let distance = distance_to_celling(status.current_collider_height);

            if distance < 0.0 {
                transform.translation -= gravity_direction * distance;
            }

            continue;
        }
    }
}

fn moving(
    time: Res<Time<Fixed>>,
    input: Res<Input>,
    mut entity_q: Query<
        (
            &mut Velocity,
            &Transform,
            &Parameters,
            &Status,
            Option<&Control>,
        ),
        Without<Floating>,
    >,
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

fn swimming(
    time: Res<Time<Fixed>>,
    input: Res<Input>,
    rapier: Single<&RapierContext, Without<Floating>>,
    mut entity_q: Query<
        (
            &mut Velocity,
            &Transform,
            &GlobalTransform,
            &Parameters,
            &Floating,
        ),
        With<Control>,
    >,
) {
    let filter = QueryFilter::default();

    for (mut velocity, transform, global_transform, parameters, floating) in entity_q.iter_mut() {
        let mut vertical_direction: f32 = 0.0;

        if input.swimming_up {
            vertical_direction += 1.0;
        }

        if input.swimming_down {
            vertical_direction -= 1.0;
        }

        let mut can_swim_up = false;

        rapier.intersections_with_point(global_transform.translation(), filter, |e| {
            can_swim_up = floating.pool() == e;
            !can_swim_up
        });

        if !can_swim_up {
            vertical_direction = vertical_direction.min(0.0);
        }

        let direction = transform.rotation
            * Vec3::new(input.moving.x, vertical_direction, input.moving.y).normalize_or_zero();

        velocity.linvel += direction * parameters.swimming_speed * time.delta_secs();
    }
}

fn falling(
    time: Res<Time<Fixed>>,
    input: Res<Input>,
    mut entity_q: Query<
        (&mut Velocity, &Transform, &Parameters, &Status),
        (With<Control>, Without<Floating>),
    >,
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
    mut input: ResMut<Input>,
    mut entity_q: Query<(&mut Velocity, &Parameters, &Status), With<Control>>,
) {
    if !input.jumping() {
        return;
    }

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
