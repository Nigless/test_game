use std::f32::consts;

use crate::animation_sequencer::{
    Animation, AnimationSequencer, AnimationSequencerSystems, Target,
};
use crate::camera_controller::CameraController;
use crate::character_body::{CharacterBody, CharacterBodySystems};
use crate::control::{Control, Input};

use bevy::utils::HashMap;

use bevy_rapier3d::dynamics::{GravityScale, Velocity};

use bevy_rapier3d::plugin::RapierConfiguration;
use bevy_rapier3d::prelude::Collider;

use bevy::prelude::*;

use crouching_falling_state::{CrouchingFallingState, CrouchingFallingStatePlugin};
use crouching_standing_state::{CrouchingStandingState, CrouchingStandingStatePlugin};
use falling_state::{FallingState, FallingStatePlugin};
use moving_state::{MovingState, MovingStatePlugin};
use running_state::{RunningState, RunningStatePlugin};
use standing_state::{StandingState, StandingStatePlugin};

mod crouching_falling_state;
mod crouching_standing_state;
mod falling_state;
mod moving_state;
mod running_state;
mod standing_state;

#[derive(Component, Default)]
struct Unresolved;

#[derive(Component, Reflect, PartialEq)]
#[reflect(Component)]
struct Stats {
    pub moving_speed: f32,
    pub jumping_high: f32,
    pub acceleration: f32,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            moving_speed: 0.025,
            jumping_high: 0.03,
            acceleration: 10.0,
        }
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

pub const COLLIDER_RADIUS: f32 = 0.3;
pub const COLLIDER_HALF_HEIGHT: f32 = 1.0 - COLLIDER_RADIUS;
pub const COLLIDER_CROUCHING_HALF_HEIGHT: f32 = COLLIDER_HALF_HEIGHT * 0.4;

#[derive(Bundle, Default)]
pub struct Ghost {
    unresolved: Unresolved,
    name: Name,
    state: StandingState,
    stats: Stats,
    transform: TransformBundle,
    velocity: Velocity,
    gravity: GravityScale,
}

impl Ghost {
    pub fn new() -> Self {
        Self {
            name: Name::new("Ghost"),
            gravity: GravityScale(1.0),
            ..default()
        }
    }
}

#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub enum GhostSystems {
    Startup,
    Resolve,
    Switch,
    Exit,
    Enter,
    Update,
}

pub struct GhostPlugin;

impl Plugin for GhostPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CrouchingFallingStatePlugin)
            .add_plugins(CrouchingStandingStatePlugin)
            .add_plugins(FallingStatePlugin)
            .add_plugins(MovingStatePlugin)
            .add_plugins(StandingStatePlugin)
            .add_plugins(RunningStatePlugin)
            .register_type::<Stats>()
            .configure_sets(Startup, GhostSystems::Startup)
            .add_systems(Startup, startup.in_set(GhostSystems::Startup))
            .configure_sets(
                PreUpdate,
                (
                    GhostSystems::Resolve,
                    GhostSystems::Switch,
                    GhostSystems::Exit,
                    GhostSystems::Enter,
                    GhostSystems::Update,
                )
                    .chain(),
            )
            .add_systems(
                PreUpdate,
                (
                    resolve.in_set(GhostSystems::Resolve),
                    (
                        look_around,
                        respawn,
                        move_standing,
                        move_falling,
                        update_head_position,
                    )
                        .in_set(GhostSystems::Update),
                ),
            );
    }
}

fn startup(mut commands: Commands, mut animations: ResMut<Assets<Animation>>) {
    let walking = Animation::new(1600).with_target(
        Target::new("camera")
            .with_property(
                "transform.rotation.x",
                vec![
                    (0, -0.002),
                    (200, 0.002),
                    (400, -0.002),
                    (600, 0.002),
                    (800, -0.002),
                    (1000, 0.002),
                    (1200, -0.002),
                    (1400, 0.002),
                ],
            )
            .with_property("transform.rotation.y", vec![(400, -0.006), (1200, 0.006)]),
    );

    commands
        .insert_resource(Animations::default().with_animation("walking", animations.add(walking)));
}

fn resolve(
    mut commands: Commands,
    animations: Res<Animations>,
    mut entity_q: Query<Entity, With<Unresolved>>,
) {
    for entity in entity_q.iter_mut() {
        let camera = commands
            .spawn((
                Name::new("camera"),
                Camera3dBundle {
                    projection: Projection::Perspective(PerspectiveProjection {
                        fov: consts::PI / 2.0,
                        ..default()
                    }),
                    ..default()
                },
            ))
            .id();

        let head = commands
            .spawn((
                Transform::from_xyz(0.0, COLLIDER_HALF_HEIGHT, 0.0),
                GlobalTransform::default(),
            ))
            .add_child(camera)
            .id();

        commands
            .entity(entity)
            .remove::<Unresolved>()
            .insert((
                CameraController::new(camera),
                AnimationSequencer::from(&animations.data).playing_all(),
                Collider::capsule_y(COLLIDER_HALF_HEIGHT, COLLIDER_RADIUS),
                CharacterBody::default(),
            ))
            .add_child(head);
    }
}

fn respawn(input: Res<Input>, mut entity_q: Query<(&mut Velocity, &mut Transform), With<Control>>) {
    if !input.pausing {
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
    mut entity_q: Query<(&mut Transform, &Children), With<Control>>,
    mut head_q: Query<&mut Transform, Without<Control>>,
) {
    if !input.is_changed() {
        return;
    }

    for (mut entity_transform, children) in entity_q.iter_mut() {
        children.get(0).unwrap();

        let mut head_transform = head_q
            .get_mut(*children.get(0).unwrap())
            .expect("character doesn't have head");

        let rotation =
            head_transform.rotation * Quat::from_rotation_x(input.looking_around.y * 0.002);

        if (rotation * Vec3::Y).y >= 0.0 {
            head_transform.rotation = rotation;
        }

        entity_transform.rotation *= Quat::from_rotation_y(input.looking_around.x * 0.002);
    }
}

fn move_standing(
    time: Res<Time>,
    rapier_config: Res<RapierConfiguration>,
    input: Res<Input>,
    mut entity_q: Query<
        (
            &mut Velocity,
            &Transform,
            &Stats,
            &CharacterBody,
            Option<&Control>,
            Option<&GravityScale>,
        ),
        Or<(
            With<StandingState>,
            With<MovingState>,
            With<RunningState>,
            With<CrouchingStandingState>,
        )>,
    >,
) {
    for (mut velocity, transform, stats, character_body, control, gravity_scale) in
        entity_q.iter_mut()
    {
        let normal = character_body.normal;

        let mut direction_along_surface = Vec3::ZERO;

        if control.is_some() {
            if input.jumping {
                let vector_jump = rapier_config.gravity.normalize_or_zero() * -stats.jumping_high;

                let vertical_speed = velocity.linvel.dot(-rapier_config.gravity);

                if vertical_speed <= 0.0 {
                    velocity.linvel += vector_jump;
                    continue;
                }

                if vertical_speed <= stats.jumping_high {
                    velocity.linvel =
                        velocity.linvel.reject_from(rapier_config.gravity) + vector_jump;
                    continue;
                }

                velocity.linvel += normal * stats.jumping_high * 0.3;

                continue;
            }

            direction_along_surface = (transform.rotation
                * Vec3::new(input.moving.y, 0.0, -input.moving.x).normalize_or_zero())
            .cross(normal);
        }

        let velocity_along_surface = velocity.linvel.reject_from(normal);

        let vector_along_surface = velocity_along_surface.lerp(
            direction_along_surface * stats.moving_speed,
            time.delta_seconds() * stats.acceleration,
        );

        velocity.linvel = vector_along_surface + velocity.linvel.project_onto(normal);

        continue;
    }
}

fn move_falling(
    time: Res<Time>,
    rapier_config: Res<RapierConfiguration>,
    input: Res<Input>,
    mut entity_q: Query<
        (
            &mut Velocity,
            &Transform,
            &Stats,
            Option<&GravityScale>,
            Option<&Control>,
        ),
        Or<(With<FallingState>, With<CrouchingFallingState>)>,
    >,
) {
    for (mut velocity, transform, stats, gravity_scale, control) in entity_q.iter_mut() {
        velocity.linvel += rapier_config.gravity
            * gravity_scale.map(|v| v.0).unwrap_or(1.0)
            * time.delta_seconds().powi(2);

        if control.is_none() || input.moving == Vec2::ZERO {
            continue;
        }

        let direction = (transform.rotation * Vec3::new(input.moving.x, 0.0, input.moving.y))
            .xz()
            .normalize_or_zero();

        let move_velocity = velocity.linvel.xz();

        let speed = stats.moving_speed * time.delta_seconds();

        let mut result = move_velocity + direction * speed;

        if result.length() > stats.moving_speed {
            let coefficient = (move_velocity.angle_between(direction) / consts::PI).abs();
            result = result.normalize_or_zero() * (move_velocity.length() - speed * coefficient);
        }

        velocity.linvel = Vec3::new(result.x, velocity.linvel.y, result.y);
    }
}

fn update_head_position(
    time: Res<Time>,
    mut entity_q: Query<
        (
            &Children,
            Option<&CrouchingStandingState>,
            Option<&CrouchingFallingState>,
        ),
        Or<(
            With<CrouchingStandingState>,
            With<CrouchingFallingState>,
            With<FallingState>,
            With<MovingState>,
            With<RunningState>,
            With<StandingState>,
        )>,
    >,
    mut head_q: Query<&mut Transform, Without<CharacterBody>>,
) {
    for (children, crouching_standing, crouching_falling) in entity_q.iter_mut() {
        let position = if crouching_standing.is_some() || crouching_falling.is_some() {
            COLLIDER_CROUCHING_HALF_HEIGHT
        } else {
            COLLIDER_HALF_HEIGHT
        };

        let entity = *children.get(0).expect("character doesn't have head");

        let mut transform = head_q.get_mut(entity).unwrap();

        transform.translation.y = transform
            .translation
            .y
            .lerp(position, time.delta_seconds() * 10.0);
    }
}
