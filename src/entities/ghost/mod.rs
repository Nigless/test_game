use std::f32::consts;
use std::slice::Windows;

use crate::bindings::{Bindings, Control};
use crate::camera_controller::{self, CameraController};

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

#[derive(Resource, Default)]
struct Input {
    pub moving: Vec2,
    pub looking: Vec2,
    pub jumping: bool,
    pub running: bool,
    pub crouching: bool,
}

#[derive(Component)]
struct Unresolved;

#[derive(Component, Reflect)]
#[reflect(Component)]
struct State {
    pub standing: bool,
    pub moving: bool,
    pub rising: bool,
    pub falling: bool,
    pub crouching: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            standing: true,
            moving: false,
            rising: false,
            falling: false,
            crouching: false,
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Characteristics {
    pub walking_speed: f32,
    pub running_speed: f32,
    pub falling_speed: f32,
    pub crouching_speed: f32,
    pub jumping_high: f32,
    pub acceleration: f32,
}

impl Default for Characteristics {
    fn default() -> Self {
        Self {
            walking_speed: 0.02,
            running_speed: 0.04,
            falling_speed: 0.01,
            crouching_speed: 0.01,
            jumping_high: 0.03,
            acceleration: 20.0,
        }
    }
}

#[derive(Bundle)]
pub struct Ghost {
    unresolved: Unresolved,
    name: Name,
    state: State,
    characteristics: Characteristics,
    transform: TransformBundle,
    collider: Collider,
    velocity: Velocity,
    gravity: GravityScale,
}

impl Ghost {
    pub fn new() -> Self {
        Self {
            unresolved: Unresolved,
            name: Name::new("Ghost"),
            state: State::default(),
            characteristics: Characteristics::default(),
            transform: TransformBundle::default(),
            collider: Collider::capsule_y(0.9, 0.2),
            velocity: Velocity::default(),
            gravity: GravityScale(1.0),
        }
    }
}

pub struct GhostPlugin;

impl Plugin for GhostPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Input::default())
            .register_type::<State>()
            .register_type::<Characteristics>()
            .add_systems(Startup, resolve)
            .add_systems(First, resolve)
            .add_systems(
                Update,
                (
                    update_input.before(look_around).before(walk),
                    look_around
                        .before(update_state)
                        .run_if(resource_changed::<Input>),
                    walk.before(update_state).run_if(resource_changed::<Input>),
                    update_state,
                    respawn,
                ),
            );
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

fn update_input(
    mut mouse: EventReader<MouseMotion>,
    keyboard: Res<ButtonInput<KeyCode>>,
    controls: Res<Bindings>,
    mut input: ResMut<Input>,
    entity_q: Query<&Control>,
) {
    if entity_q.is_empty() {
        return;
    }

    input.looking = Vec2::ZERO;

    for event in mouse.read().into_iter() {
        input.looking += Vec2::new(-event.delta.x, -event.delta.y);
    }

    input.moving = Vec2::ZERO;

    if keyboard.pressed(controls.move_left) {
        input.moving += Vec2::new(-1.0, 0.0);
    }

    if keyboard.pressed(controls.move_right) {
        input.moving += Vec2::new(1.0, 0.0);
    }

    if keyboard.pressed(controls.move_forward) {
        input.moving += Vec2::new(0.0, -1.0);
    }

    if keyboard.pressed(controls.move_backward) {
        input.moving += Vec2::new(0.0, 1.0);
    }

    input.jumping = keyboard.just_pressed(controls.jump);
    input.running = keyboard.pressed(controls.run);
    input.crouching = keyboard.pressed(controls.crouch);
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
    mut window_q: Query<&mut Window>,
    mut entity_q: Query<(&mut Transform, &CameraController), (Without<Unresolved>, With<Control>)>,
    mut camera_q: Query<&mut Transform, (With<Camera>, Without<CameraController>)>,
) {
    // let mut window = window_q.single_mut();
    // let width = window.width();
    // let height = window.height();
    // window.set_cursor_position(Some(Vec2::new(width / 2., height / 2.)));
    // window.cursor.visible = false;

    for (mut entity_transform, controller) in entity_q.iter_mut() {
        let mut camera_transform = camera_q
            .get_mut(controller.target)
            .expect("CameraController target doesn't exist or doesn't have a Camera component");

        let rotation = camera_transform.rotation * Quat::from_rotation_x(input.looking.y * 0.002);

        if (rotation * Vec3::Y).y > 0.0 {
            camera_transform.rotation = rotation;
        }

        entity_transform.rotation *= Quat::from_rotation_y(input.looking.x * 0.002);
    }
}

fn walk(
    time: Res<Time>,
    input: Res<Input>,
    mut entity_q: Query<(&mut Velocity, &mut Transform, &State, &Characteristics), With<Control>>,
) {
    for (mut velocity, mut transform, state, characteristics) in entity_q.iter_mut() {
        let is_grounded = state.standing || state.moving || state.crouching;

        if input.jumping && is_grounded {
            velocity.linvel.y = characteristics.jumping_high;
        }

        let max_speed = if state.standing || state.moving {
            if input.running {
                characteristics.running_speed
            } else {
                characteristics.walking_speed
            }
        } else if state.falling || state.rising {
            characteristics.falling_speed
        } else if state.crouching {
            characteristics.crouching_speed
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

        let move_velocity = velocity.linvel.xz();

        if is_grounded {
            if input.jumping {
                velocity.linvel.y = characteristics.jumping_high;
            }

            let result = move_velocity.lerp(
                direction * max_speed,
                characteristics.acceleration * time.delta_seconds(),
            );

            velocity.linvel = Vec3::new(result.x, velocity.linvel.y, result.y);

            continue;
        }

        if direction == Vec2::ZERO {
            continue;
        }

        let speed = max_speed * time.delta_seconds();

        let mut result = move_velocity + direction * speed;

        if result.length() > max_speed {
            let coefficient = (move_velocity.angle_between(direction) / consts::PI).abs();
            result = result.normalize_or_zero() * (move_velocity.length() - speed * coefficient);
        }

        velocity.linvel = Vec3::new(result.x, velocity.linvel.y, result.y);
    }
}

fn collide_and_slide(
    rapier: &Res<RapierContext>,
    rotation: Quat,
    collider: &Collider,
    entity: Entity,
    gravity: Vec3,
    mut velocity: Vec3,
    position: Vec3,
) -> (Vec3, Vec3) {
    let skin_width = 0.1;

    let mut vector_cast = velocity + gravity;
    vector_cast = vector_cast.normalize_or_zero() * (vector_cast.length() + skin_width);
    let collision = rapier
        .cast_shape(
            position,
            rotation,
            vector_cast,
            collider,
            1.0,
            true,
            QueryFilter::new().exclude_collider(entity),
        )
        .map_or(None, |(_, hit)| {
            hit.details.map(|details| (hit.toi, details))
        });

    if let Some((time_of_impact, details)) = collision {
        let normal = details.normal1.normalize_or_zero();

        let mut vector_to_surface =
            vector_cast.normalize_or_zero() * (vector_cast.length() * time_of_impact - skin_width);

        if vector_to_surface.length() <= skin_width {
            vector_to_surface = Vec3::ZERO;
        }

        if gravity.angle_between(normal) < consts::PI * 0.75 {
            velocity = velocity + gravity - vector_to_surface
        }

        let vector_slide = velocity.reject_from(normal);

        let (vector_result, _) = collide_and_slide(
            rapier,
            rotation,
            collider,
            entity,
            Vec3::ZERO,
            vector_slide,
            position + vector_to_surface,
        );

        return (vector_to_surface + vector_result, normal);
    };

    return (velocity + gravity, Vec3::ZERO);
}

fn update_state(
    time: Res<Time>,
    rapier: Res<RapierContext>,
    rapier_config: Res<RapierConfiguration>,
    mut entity_q: Query<(
        Entity,
        &mut State,
        &mut Velocity,
        &mut Transform,
        &Collider,
        &GravityScale,
    )>,
) {
    for (entity, mut state, mut velocity, mut transform, collider, gravity) in entity_q.iter_mut() {
        let rotation = transform.rotation;
        let position = transform.translation;
        let vector_gravity = rapier_config.gravity * gravity.0 * time.delta_seconds().powi(2);

        let (corrected_velocity, normal) = collide_and_slide(
            &rapier,
            rotation,
            collider,
            entity,
            vector_gravity,
            velocity.linvel,
            position,
        );

        let mut grounded = false;

        if vector_gravity.angle_between(normal) > consts::PI * 0.75 {
            grounded = true;
        }

        velocity.linvel = corrected_velocity;

        transform.translation += velocity.linvel;

        if grounded {
            let speed = velocity.linvel.xz().length();

            if state.standing && speed > 0.001 {
                state.standing = false;
                state.moving = true;
                continue;
            }

            if state.moving && speed < 0.001 {
                state.moving = false;
                state.standing = true;
                continue;
            }

            if state.rising && speed > 0.001 {
                state.rising = false;
                state.moving = true;
                continue;
            }

            if state.rising && speed < 0.001 {
                state.rising = false;
                state.standing = true;
                continue;
            }

            if state.falling && speed > 0.001 {
                state.falling = false;
                state.moving = true;
                continue;
            }

            if state.falling && speed < 0.001 {
                state.falling = false;
                state.standing = true;
                continue;
            }

            continue;
        };

        if state.standing && velocity.linvel.y > 0.0 {
            state.standing = false;
            state.rising = true;
            continue;
        }

        if state.moving && velocity.linvel.y > 0.0 {
            state.moving = false;
            state.rising = true;
            continue;
        }

        if state.rising && velocity.linvel.y < 0.0 {
            state.rising = false;
            state.falling = true;
            continue;
        }

        if state.falling && velocity.linvel.y > 0.0 {
            state.falling = false;
            state.rising = true;
            continue;
        }

        if state.standing {
            state.standing = false;
            state.falling = true;
            continue;
        }

        if state.moving {
            state.moving = false;
            state.falling = true;
            continue;
        }
    }
}
