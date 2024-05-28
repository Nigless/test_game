use std::f32::consts;
use std::slice::Windows;

use crate::camera_controller::{self, CameraController};
use crate::character_body::CharacterBody;
use crate::control::{Bindings, Control, Input};

use bevy::log::tracing_subscriber::filter;
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

use self::crouching_state::{CrouchingState, CrouchingStatePlugin};
use self::falling_state::{FallingState, FallingStatePlugin};
use self::moving_state::{MovingState, MovingStatePlugin};
use self::rising_state::{RisingState, RisingStatePlugin};
use self::standing_state::{StandingState, StandingStatePlugin};
mod crouching_state;
mod falling_state;
mod moving_state;
mod rising_state;
mod standing_state;

#[derive(Component)]
struct Unresolved;

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
struct Stats {
    pub walking_speed: f32,
    pub running_speed: f32,
    pub falling_speed: f32,
    pub crouching_speed: f32,
    pub jumping_high: f32,
    pub crouching_jump_high: f32,
    pub acceleration: f32,
}

#[derive(Bundle)]
pub struct Ghost {
    unresolved: Unresolved,
    name: Name,
    state: StandingState,
    stats: Stats,
    transform: TransformBundle,
    velocity: Velocity,
    collider: Collider,
    character_body: CharacterBody,
}

impl Ghost {
    pub fn new() -> Self {
        Self {
            unresolved: Unresolved,
            name: Name::new("Ghost"),
            state: StandingState,
            stats: Stats::default(),
            transform: TransformBundle::default(),
            collider: Collider::capsule_y(0.9, 0.2),
            character_body: CharacterBody::default(),
            velocity: Velocity::default(),
        }
    }
}

pub struct GhostPlugin;

impl Plugin for GhostPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            StandingStatePlugin,
            MovingStatePlugin,
            RisingStatePlugin,
            FallingStatePlugin,
            CrouchingStatePlugin,
        ))
        .register_type::<Stats>()
        .add_systems(Startup, resolve)
        .add_systems(First, resolve)
        .add_systems(Update, (look_around, move_char, respawn));
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
    mut entity_q: Query<(&mut Transform, &CameraController), (Without<Unresolved>, With<Control>)>,
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

fn move_char(
    time: Res<Time>,
    input: Res<Input>,
    mut entity_q: Query<
        (
            &mut Velocity,
            &Transform,
            &Stats,
            &CharacterBody,
            Option<&StandingState>,
            Option<&MovingState>,
            Option<&CrouchingState>,
            Option<&FallingState>,
            Option<&RisingState>,
        ),
        With<Control>,
    >,
) {
    for (
        mut velocity,
        transform,
        stats,
        character_body,
        standing,
        moving,
        crouching,
        falling,
        rising,
    ) in entity_q.iter_mut()
    {
        let standing = standing.is_some();
        let moving = moving.is_some();
        let crouching = crouching.is_some();
        let falling = falling.is_some();
        let rising = rising.is_some();

        let max_speed = if standing || moving {
            if input.running {
                stats.running_speed
            } else {
                stats.walking_speed
            }
        } else if falling || rising {
            stats.falling_speed
        } else if crouching {
            stats.crouching_speed
        } else {
            continue;
        };

        let direction = if input.moving == Vec2::ZERO {
            Vec2::ZERO
        } else {
            (transform.rotation * Vec3::new(input.moving.x, 0.0, input.moving.y))
                .xz()
                .normalize_or_zero()
        };

        if character_body.is_grounded {
            let move_velocity = velocity.linvel.xz();

            let jump_high = if crouching {
                stats.crouching_jump_high
            } else {
                stats.jumping_high
            };

            if input.jumping {
                velocity.linvel.y = jump_high;
            }

            let result = move_velocity.lerp(
                direction * max_speed,
                stats.acceleration * time.delta_seconds(),
            );

            velocity.linvel = Vec3::new(result.x, velocity.linvel.y, result.y);

            continue;
        }

        if direction == Vec2::ZERO {
            continue;
        }

        let direction = (transform.rotation * Vec3::new(input.moving.x, 0.0, input.moving.y))
            .xz()
            .normalize_or_zero();

        let move_velocity = velocity.linvel.xz();

        let speed = max_speed * time.delta_seconds();

        let mut result = move_velocity + direction * speed;

        if result.length() > max_speed {
            let coefficient = (move_velocity.angle_between(direction) / consts::PI).abs();
            result = result.normalize_or_zero() * (move_velocity.length() - speed * coefficient);
        }

        velocity.linvel = Vec3::new(result.x, velocity.linvel.y, result.y);
    }
}
