use std::f32::consts;

use crate::animation_sequencer::{
    Animation, AnimationSequencer, AnimationSequencerSystems, Target,
};
use crate::camera_controller::CameraController;
use crate::character_body::CharacterBody;
use crate::control::{Control, Input};
use crate::lib::move_toward;

use bevy::utils::HashMap;

use bevy_rapier3d::dynamics::{GravityScale, Velocity};

use bevy_rapier3d::na::Quaternion;
use bevy_rapier3d::plugin::RapierConfiguration;
use bevy_rapier3d::prelude::Collider;

use bevy::prelude::*;

#[derive(Component, Default)]
struct Unresolved;

#[derive(Component, Reflect, PartialEq)]
#[reflect(Component)]
struct Parameters {
    pub walking_speed: f32,
    pub standing_acceleration: f32,
    pub standing_jump_height: f32,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            walking_speed: 4.0,
            standing_acceleration: 0.4,
            standing_jump_height: 2.5,
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
pub struct GhostBundle {
    unresolved: Unresolved,
    name: Name,
    parameters: Parameters,
    transform: TransformBundle,
    velocity: Velocity,
    gravity: GravityScale,
    collider: Collider,
    body: CharacterBody,
}

impl GhostBundle {
    pub fn new() -> Self {
        Self {
            name: Name::new("Ghost"),
            collider: Collider::capsule_y(COLLIDER_HALF_HEIGHT, COLLIDER_RADIUS),
            ..default()
        }
    }
}

#[derive(Bundle, Default)]
pub struct GhostCamera {
    name: Name,
    camera: Camera3dBundle,
}

impl GhostCamera {
    pub fn new() -> Self {
        Self {
            name: Name::new("camera"),
            camera: Camera3dBundle {
                projection: Projection::Perspective(PerspectiveProjection {
                    fov: consts::PI / 2.0,
                    ..default()
                }),
                ..default()
            },
        }
    }
}

pub struct GhostPlugin;

impl Plugin for GhostPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Parameters>()
            .add_systems(Startup, startup)
            .add_systems(First, resolve)
            .add_systems(
                PreUpdate,
                (
                    looking.run_if(looking_condition),
                    gravity,
                    moving,
                    jumping.run_if(jumping_condition),
                )
                    .chain(),
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
        let camera = commands.spawn(GhostCamera::new()).id();

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
            ))
            .add_child(head);
    }
}

fn looking_condition(input: Res<Input>) -> bool {
    input.is_changed()
}

fn looking(
    input: Res<Input>,
    mut entity_q: Query<(&mut Transform, &Children), (With<Control>, With<Parameters>)>,
    mut head_q: Query<&mut Transform, Without<Parameters>>,
) {
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

fn gravity(
    mut entity_q: Query<(&mut Velocity, Option<&GravityScale>), With<Parameters>>,
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
    mut entity_q: Query<(&mut Velocity, &Transform, &Parameters), With<Control>>,
) {
    for (mut velocity, transform, parameters) in entity_q.iter_mut() {
        let ground_surface = Vec3::Y;
        let direction =
            transform.rotation * Vec3::new(input.moving.x, 0.0, input.moving.y).normalize_or_zero();

        let vertical_velocity = velocity.linvel.project_onto(ground_surface);

        let mut horizontal_velocity = velocity.linvel - vertical_velocity;

        horizontal_velocity = move_toward(
            horizontal_velocity,
            direction * parameters.walking_speed,
            parameters.standing_acceleration,
        );

        velocity.linvel = horizontal_velocity + vertical_velocity;
    }
}

fn jumping_condition(input: Res<Input>, entity_q: Query<&Control>) -> bool {
    input.jumping && !entity_q.is_empty()
}

fn jumping(mut entity_q: Query<(&mut Velocity, &Parameters), With<Control>>) {
    for (mut velocity, parameters) in entity_q.iter_mut() {
        let ground_surface = Vec3::Y;

        velocity.linvel = velocity.linvel - velocity.linvel.project_onto(ground_surface)
            + ground_surface * parameters.standing_jump_height;
    }
}
