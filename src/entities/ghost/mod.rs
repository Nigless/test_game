use std::f32::consts;
use std::fmt::Display;
use std::slice::Windows;

use crate::camera_controller::{self, CameraController};
use crate::character_body::CharacterBody;
use crate::control::{Bindings, Control, Input};

use bevy::log::tracing_subscriber::filter;
use bevy::scene::ron::value;
use bevy::{ecs::reflect, input::mouse::MouseMotion};
use bevy_rapier3d::control::{self, CharacterLength};
use bevy_rapier3d::dynamics::{GravityScale, RigidBody, Velocity};
use bevy_rapier3d::geometry::RapierColliderHandle;
use bevy_rapier3d::na::Isometry;
use bevy_rapier3d::pipeline::QueryFilter;
use bevy_rapier3d::plugin::{RapierConfiguration, RapierContext};
use bevy_rapier3d::prelude::Collider;

use bevy::{prelude::*, transform};
use bevy_rapier3d::prelude::LockedAxes;
use bevy_rapier3d::rapier::control::KinematicCharacterController;

#[derive(Component, Default)]
struct Unresolved;

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
struct Stats {
    pub moving_speed: f32,
    pub jumping_high: f32,
    pub acceleration: f32,
}

#[derive(Component, Default, Reflect, Debug)]
#[reflect(Component)]
pub enum GhostState {
    #[default]
    Standing,
    Moving,
    Rising,
    Falling,
    Crouching,
    Running,
}

#[derive(Bundle, Default)]
pub struct Ghost {
    unresolved: Unresolved,
    name: Name,
    state: GhostState,
    stats: Stats,
    transform: TransformBundle,
    velocity: Velocity,
    collider: Collider,
    character_body: CharacterBody,
}

impl Ghost {
    pub fn new() -> Self {
        Self {
            name: Name::new("Ghost"),
            collider: Collider::capsule_y(0.9, 0.2),
            ..default()
        }
    }
}

pub struct GhostPlugin;

impl Plugin for GhostPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Stats>()
            .register_type::<GhostState>()
            .add_systems(Startup, resolve)
            .add_systems(First, resolve)
            .add_systems(Update, (look_around, move_character, respawn))
            .add_systems(PostUpdate, update_state)
            .add_systems(PostUpdate, update_stats);
    }
}

fn resolve(mut entity_q: Query<Entity, With<Unresolved>>, mut commands: Commands) {
    let projection = Projection::Perspective(PerspectiveProjection {
        fov: consts::PI / 2.0,
        ..default()
    });

    let transform = Transform::from_xyz(0.0, 0.9 - 0.2, 0.0);

    for entity in entity_q.iter_mut() {
        let camera = commands
            .spawn(Camera3dBundle {
                transform,
                projection: projection.clone(),
                ..default()
            })
            .id();

        commands
            .entity(entity)
            .remove::<Unresolved>()
            .insert(CameraController { target: camera })
            .add_child(camera);
    }
}

fn update_state(
    input: Res<Input>,
    mut entity_q: Query<(&Velocity, &CharacterBody, &mut GhostState)>,
) {
    for (velocity, character_body, mut state) in entity_q.iter_mut() {
        let is_moving = velocity.linvel.xz().length() > 0.01;

        if input.crouching {
            *state = GhostState::Crouching;

            continue;
        }

        if character_body.is_grounded {
            if is_moving {
                if input.running {
                    *state = GhostState::Running;

                    continue;
                }

                *state = GhostState::Moving;

                continue;
            }
            *state = GhostState::Standing;

            continue;
        }

        if velocity.linvel.y > 0.01 {
            *state = GhostState::Rising;

            continue;
        }

        if velocity.linvel.y < -0.01 {
            *state = GhostState::Falling;
        }
    }
}

fn update_stats(mut entity_q: Query<(&GhostState, &mut Stats), Changed<GhostState>>) {
    for (state, mut stats) in entity_q.iter_mut() {
        *stats = match state {
            GhostState::Standing => Stats {
                moving_speed: 0.02,
                jumping_high: 0.03,
                acceleration: 20.0,
            },
            GhostState::Moving => Stats {
                moving_speed: 0.02,
                jumping_high: 0.03,
                acceleration: 20.0,
            },
            GhostState::Rising => Stats {
                moving_speed: 0.01,
                jumping_high: 0.03,
                acceleration: 20.0,
            },
            GhostState::Falling => Stats {
                moving_speed: 0.01,
                jumping_high: 0.03,
                acceleration: 20.0,
            },
            GhostState::Crouching => Stats {
                moving_speed: 0.01,
                jumping_high: 0.02,
                acceleration: 20.0,
            },
            GhostState::Running => Stats {
                moving_speed: 0.04,
                jumping_high: 0.03,
                acceleration: 20.0,
            },
        };
    }
}

fn respawn(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut entity_q: Query<(&mut Velocity, &mut Transform), With<Control>>,
) {
    if !keyboard.just_pressed(KeyCode::Escape) {
        return;
    }

    for (mut velocity, mut transform) in entity_q.iter_mut() {
        velocity.linvel = Vec3::ZERO;
        velocity.angvel = Vec3::ZERO;
        transform.translation = Vec3::Y * 2.0;
        transform.rotation = Quat::IDENTITY;
    }
}

fn look_around(
    input: Res<Input>,
    mut entity_q: Query<(&mut Transform, &CameraController), With<Control>>,
    mut camera_q: Query<&mut Transform, (With<Camera>, Without<CameraController>)>,
) {
    for (mut entity_transform, controller) in entity_q.iter_mut() {
        let mut camera_transform = camera_q
            .get_mut(controller.target)
            .expect("CameraController target doesn't exist or doesn't have a Camera component");

        let rotation =
            camera_transform.rotation * Quat::from_rotation_x(input.looking_around.y * 0.002);

        if (rotation * Vec3::Y).y >= 0.0 {
            camera_transform.rotation = rotation;
        }

        entity_transform.rotation *= Quat::from_rotation_y(input.looking_around.x * 0.002);
    }
}

fn move_character(
    time: Res<Time>,
    rapier_config: Res<RapierConfiguration>,
    input: Res<Input>,
    mut entity_q: Query<
        (
            &mut Velocity,
            &Transform,
            &Stats,
            &CharacterBody,
            Option<&GravityScale>,
        ),
        With<Control>,
    >,
) {
    for (mut velocity, transform, stats, character_body, gravity_scale) in entity_q.iter_mut() {
        let direction = if input.moving == Vec2::ZERO {
            Vec2::ZERO
        } else {
            (transform.rotation * Vec3::new(input.moving.x, 0.0, input.moving.y))
                .xz()
                .normalize_or_zero()
        };

        let mut move_velocity = velocity.linvel.xz();

        if character_body.is_grounded {
            move_velocity = move_velocity.lerp(
                direction * stats.moving_speed,
                stats.acceleration * time.delta_seconds(),
            );

            let mut vertical_velocity = velocity.linvel.y;

            if input.jumping {
                vertical_velocity = stats.jumping_high;
            }

            velocity.linvel = Vec3::new(move_velocity.x, vertical_velocity, move_velocity.y);

            continue;
        }

        velocity.linvel += rapier_config.gravity
            * gravity_scale.map(|v| v.0).unwrap_or(1.0)
            * time.delta_seconds().powi(2);

        if direction == Vec2::ZERO {
            continue;
        }

        let speed = stats.moving_speed * time.delta_seconds();

        let mut result = move_velocity + direction * speed;

        if result.length() > stats.moving_speed {
            let coefficient = (move_velocity.angle_between(direction) / consts::PI).abs();
            result = result.normalize_or_zero() * (move_velocity.length() - speed * coefficient);
        }

        velocity.linvel = Vec3::new(result.x, velocity.linvel.y, result.y);
    }
}
