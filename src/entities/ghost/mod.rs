use std::f32::consts;
use std::fmt::Display;
use std::ops::Deref;
use std::slice::Windows;
use std::time::Duration;

use crate::animation_sequencer::{Animation, AnimationSequencer, Keyframe, Sequence};
use crate::camera_controller::{self, CameraController};
use crate::character_body::{self, CharacterBody};
use crate::control::{Bindings, Control, Input};

use bevy::log::tracing_subscriber::filter;
use bevy::scene::ron::value;
use bevy::utils::HashMap;
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

#[derive(Component, Default, Reflect, PartialEq)]
#[reflect(Component)]
struct Stats {
    pub moving_speed: f32,
    pub jumping_high: f32,
    pub acceleration: f32,
}

#[derive(Component, Default, Reflect, Debug, PartialEq)]
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

impl GhostState {
    fn update(&mut self, state: GhostState) {
        if *self == state {
            return;
        }

        *self = state
    }
}

#[derive(Resource, Default)]
struct Animations {
    data: HashMap<String, Handle<Animation>>,
}

impl Animations {
    fn with_animation(mut self, name: &str, animation: Handle<Animation>) -> Self {
        self.data.insert(name.to_owned(), animation);

        self
    }

    fn get(&self, name: &str) -> Option<&Handle<Animation>> {
        self.data.get(name)
    }
}

#[derive(Bundle, Default)]
pub struct Ghost {
    unresolved: Unresolved,
    name: Name,
    state: GhostState,
    stats: Stats,
    transform: TransformBundle,
    velocity: Velocity,
}

impl Ghost {
    pub fn new() -> Self {
        Self {
            name: Name::new("Ghost"),
            ..default()
        }
    }
}

pub struct GhostPlugin;

impl Plugin for GhostPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Stats>()
            .register_type::<GhostState>()
            .add_systems(Startup, (startup.before(resolve), resolve))
            .add_systems(First, resolve)
            .add_systems(Update, (look_around, move_character, respawn))
            .add_systems(PostUpdate, (update_state, update_stats, update_animation));
    }
}

fn startup(mut commands: Commands, mut animations: ResMut<Assets<Animation>>) {
    let crouching =
        Animation::new(0).with_property("transform.scale.y", vec![Keyframe::new(0, 0.5)]);
    let standing =
        Animation::new(0).with_property("transform.scale.y", vec![Keyframe::new(0, 1.0)]);

    commands.insert_resource(
        Animations::default()
            .with_animation("crouching", animations.add(crouching))
            .with_animation("standing", animations.add(standing)),
    );
}

fn resolve(
    mut commands: Commands,
    animations: Res<Animations>,
    mut entity_q: Query<Entity, With<Unresolved>>,
) {
    for entity in entity_q.iter_mut() {
        let camera = commands
            .spawn(Camera3dBundle {
                transform: Transform::from_xyz(0.0, 0.9 - 0.2, 0.0),
                projection: Projection::Perspective(PerspectiveProjection {
                    fov: consts::PI / 2.0,
                    ..default()
                }),
                ..default()
            })
            .id();

        let collider = commands
            .spawn((
                Name::new("collider"),
                Collider::capsule_y(0.9, 0.2),
                TransformBundle::default(),
                AnimationSequencer::default()
                    .with_sequence(
                        "standing",
                        Sequence::from(animations.get("standing").unwrap())
                            .with_weight(1.0)
                            .playing(),
                    )
                    .with_sequence(
                        "crouching",
                        Sequence::from(animations.get("crouching").unwrap())
                            .with_weight(0.0)
                            .playing(),
                    ),
            ))
            .id();

        commands
            .entity(entity)
            .remove::<Unresolved>()
            .insert((CameraController::new(camera), CharacterBody::new(collider)))
            .add_child(camera)
            .add_child(collider);
    }
}

fn update_state(
    input: Res<Input>,
    mut entity_q: Query<(&Velocity, &CharacterBody, &mut GhostState)>,
) {
    for (velocity, character_body, mut state) in entity_q.iter_mut() {
        let is_moving = velocity.linvel.xz().length() > 0.01;

        let mut update = |new_state: GhostState| {
            if *state != new_state {
                *state = new_state;
            }
        };

        if input.crouching {
            update(GhostState::Crouching);

            continue;
        }

        if character_body.is_grounded {
            if is_moving {
                if input.running {
                    update(GhostState::Running);

                    continue;
                }

                update(GhostState::Moving);

                continue;
            }

            update(GhostState::Standing);

            continue;
        }

        if velocity.linvel.y > 0.0 {
            update(GhostState::Rising);

            continue;
        }

        if velocity.linvel.y <= 0.0 {
            update(GhostState::Falling);

            continue;
        }
    }
}

fn update_stats(mut entity_q: Query<(&GhostState, &mut Stats), Changed<GhostState>>) {
    for (state, mut stats) in entity_q.iter_mut() {
        let mut result = Stats::default();

        result.moving_speed = 0.025;
        result.jumping_high = 0.03;
        result.acceleration = 20.0;

        match state {
            GhostState::Standing => (),
            GhostState::Moving => (),
            GhostState::Rising => result.moving_speed = 0.015,
            GhostState::Falling => result.moving_speed = 0.015,
            GhostState::Crouching => {
                result.moving_speed = 0.015;
                result.jumping_high = 0.02;
            }
            GhostState::Running => result.moving_speed = 0.04,
        };

        if *stats != result {
            *stats = result;
        }
    }
}

fn update_animation(
    mut commands: Commands,
    entity_q: Query<(&GhostState, &CharacterBody), Changed<GhostState>>,
    mut collider_q: Query<(Entity, &mut AnimationSequencer)>,
) {
    for (state, character_body) in entity_q.iter() {
        let (entity, mut sequencer) = collider_q.get_mut(character_body.collider).unwrap();

        match state {
            GhostState::Crouching => {
                sequencer.add_transition("crouching", 1.0, 300);
                sequencer.add_transition("standing", 0.0, 300);
            }
            _ => {
                sequencer.add_transition("crouching", 0.0, 300);
                sequencer.add_transition("standing", 1.0, 300);
            }
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
    if !input.is_changed() {
        return;
    }

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

        if character_body.is_grounded {
            let normal = character_body.normal;

            let direction_along_surface = (transform.rotation
                * Vec3::new(input.moving.x, 0.0, input.moving.y))
            .reject_from(normal)
            .normalize_or_zero();

            let velocity_along_surface = velocity.linvel.reject_from(normal);

            let vector_along_surface = velocity_along_surface.lerp(
                direction_along_surface * stats.moving_speed,
                stats.acceleration * time.delta_seconds(),
            );

            velocity.linvel = vector_along_surface + velocity.linvel.project_onto(normal);

            if input.jumping {
                velocity.linvel +=
                    rapier_config.gravity.normalize_or_zero() * -1.0 * stats.jumping_high
            }

            continue;
        }

        let move_velocity = velocity.linvel.xz();

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
