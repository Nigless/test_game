use std::f32::{consts, INFINITY};

use crate::camera_controller::CameraController;
use crate::control::{Control, ControlSystems, Input};
use crate::library::move_toward;
use crate::linker::Linker;
use crate::ray_caster::RayCasterSystems;
use crate::shape_caster::{ShapeCaster, ShapeCasterSystems};

use bevy_rapier3d::dynamics::Velocity;

use bevy_rapier3d::plugin::RapierConfiguration;
use bevy_rapier3d::prelude::{Collider, GravityScale};

use bevy::prelude::*;
use components::{Parameters, Status, Unresolved};
use entities::{GhostCamera, Head, RayCast, ShapeCast};
mod components;
mod entities;
pub use entities::GhostBundle;

pub const MAX_SLOPE_ANGLE: f32 = consts::PI / 3.8;
pub const HAND_DISTANCE: f32 = 2.0;
pub const COLLIDER_TRANSITION_SPEED: f32 = 0.1;
pub const COLLIDER_RADIUS: f32 = 0.3;
pub const MAX_SURFACE_GAP: f32 = 0.03 + SKIN_WIDTH;
pub const SKIN_WIDTH: f32 = 0.05;
pub const COLLIDER_HALF_HEIGHT: f32 = 1.0 - COLLIDER_RADIUS;
pub const COLLIDER_CROUCHING_HALF_HEIGHT: f32 = COLLIDER_HALF_HEIGHT * 0.4;

#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub enum GhostSystems {
    Resolve,
    Update,
    Prepare,
    FixedUpdate,
}

pub struct GhostPlugin;

impl Plugin for GhostPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Parameters>()
            .register_type::<Status>()
            .add_systems(First, resolve.in_set(GhostSystems::Resolve))
            .add_systems(
                PreUpdate,
                camera
                    .in_set(GhostSystems::Update)
                    .after(ControlSystems)
                    .before(RayCasterSystems),
            )
            .configure_sets(
                FixedPreUpdate,
                (GhostSystems::Prepare, GhostSystems::FixedUpdate)
                    .chain()
                    .after(ShapeCasterSystems),
            )
            .add_systems(
                FixedPreUpdate,
                (
                    ground_check.in_set(GhostSystems::Prepare),
                    (
                        collider,
                        moving,
                        falling.run_if(|input: Res<Input>| input.moving.length() > 0.0),
                        jumping,
                    )
                        .in_set(GhostSystems::FixedUpdate),
                ),
            );
    }
}

fn resolve(mut commands: Commands, mut entity_q: Query<(Entity, &Transform), With<Unresolved>>) {
    for (entity, transform) in entity_q.iter_mut() {
        let camera = commands.spawn(GhostCamera::new()).id();

        let ray_cast = commands.spawn(RayCast::new(entity)).id();

        let head = commands
            .spawn(Head::new(Vec3::Y * COLLIDER_HALF_HEIGHT))
            .add_child(camera)
            .add_child(ray_cast)
            .id();

        let cast_up = commands.spawn(ShapeCast::up(entity)).id();
        let cast_down = commands.spawn(ShapeCast::down(entity)).id();

        commands
            .entity(entity)
            .remove::<Unresolved>()
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
            .add_child(cast_down);
    }
}

fn ground_check(
    mut entity_q: Query<(
        &Linker,
        &mut Status,
        &mut GravityScale,
        &mut Transform,
        &mut Velocity,
    )>,
    time: Res<Time>,
    caster_q: Query<&ShapeCaster, Without<Status>>,
    config_q: Query<&RapierConfiguration, Without<ShapeCaster>>,
) {
    let config = config_q.get_single().unwrap();

    for (linker, mut status, mut gravity, mut transform, mut velocity) in entity_q.iter_mut() {
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
    time: Res<Time>,
    mut entity_q: Query<(&mut Collider, &mut Status, &mut Transform, &Linker), With<Control>>,
    caster_q: Query<&ShapeCaster, Without<Status>>,
    mut head_q: Query<&mut Transform, Without<Status>>,
    config_q: Query<&RapierConfiguration, Without<ShapeCaster>>,
) {
    let config = config_q.get_single().unwrap();

    for (mut collider, mut status, mut transform, linker) in entity_q.iter_mut() {
        let cast_down = caster_q.get(*linker.get("cast_down").unwrap()).unwrap();
        let cast_up = caster_q.get(*linker.get("cast_up").unwrap()).unwrap();

        let is_touching_ground = |height: f32| {
            if let Some(result) = cast_down.result.as_ref() {
                let surface_gap = result.distance - SKIN_WIDTH - height;

                return surface_gap < MAX_SURFACE_GAP;
            }
            false
        };

        let celling_gap = |height: f32| {
            if let Some(result) = cast_up.result.as_ref() {
                let surface_gap = result.distance - SKIN_WIDTH - height;

                return surface_gap < SKIN_WIDTH;
            }
            false
        };

        let get_height_diff = || {
            let target_height = if input.crouching {
                COLLIDER_CROUCHING_HALF_HEIGHT
            } else if status.can_standup {
                COLLIDER_HALF_HEIGHT
            } else {
                return 0.0;
            };

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

        if status.current_collider_height < original_height {
            if is_touching_ground(original_height) {
                transform.translation -= gravity_direction * height_diff;
            }
            continue;
        }

        if status.current_collider_height > original_height {
            if is_touching_ground(status.current_collider_height) {
                transform.translation -= gravity_direction * height_diff;

                continue;
            }

            if celling_gap(status.current_collider_height) {
                transform.translation += gravity_direction * height_diff;
            }

            continue;
        }
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
