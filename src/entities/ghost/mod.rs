use std::f32::consts;

use crate::bindings::{Bindings, Control};
use crate::camera_controller::{self, CameraController};

use bevy::{ecs::reflect, input::mouse::MouseMotion};
use bevy_rapier3d::control::{
    self, CharacterLength, KinematicCharacterController, KinematicCharacterControllerOutput,
};
use bevy_rapier3d::dynamics::{GravityScale, RigidBody, Velocity};
use bevy_rapier3d::na::Isometry;
use bevy_rapier3d::pipeline::QueryFilter;
use bevy_rapier3d::plugin::RapierContext;
use bevy_rapier3d::prelude::Collider;

use bevy::{prelude::*, transform};
use bevy_rapier3d::prelude::LockedAxes;
use bevy_rapier3d::rapier::geometry::ColliderSet;
use bevy_rapier3d::rapier::pipeline::QueryPipeline;

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
            walking_speed: 0.8,
            running_speed: 2.5,
            falling_speed: 2.0,
            crouching_speed: 2.0,
            jumping_high: 3.0,
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
                ),
            );
    }
}

fn resolve(mut entity_q: Query<Entity, With<Unresolved>>, mut commands: Commands) {
    let projection = Projection::Perspective(PerspectiveProjection {
        fov: consts::PI / 2.0,
        ..default()
    });

    let transform = Transform::from_xyz(0.0, 1.8 - 0.2, 0.0);

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

fn look_around(
    input: Res<Input>,
    mut entity_q: Query<
        (&mut Transform, &CameraController, &State),
        (Without<Unresolved>, With<Control>),
    >,
    mut camera_q: Query<&mut Transform, (With<Camera>, Without<CameraController>)>,
) {
    for (mut entity_transform, controller, state) in entity_q.iter_mut() {
        let mut camera_transform = camera_q
            .get_mut(controller.target)
            .expect("CameraController target doesn't exist or doesn't have a Camera component");

        camera_transform.rotation *= Quat::from_rotation_x(input.looking.y * 0.002);
        entity_transform.rotation *= Quat::from_rotation_y(input.looking.x * 0.002);
    }
}

fn walk(
    time: Res<Time>,
    input: Res<Input>,
    mut entity_q: Query<(&mut Velocity, &mut Transform, &State, &Characteristics), With<Control>>,
) {
    for (mut velocity, mut transform, state, characteristics) in entity_q.iter_mut() {
        let speed = if state.standing || state.moving {
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
            0.0
        };

        let direction = transform
            .rotation
            .mul_vec3(Vec3::new(input.moving.x, 0.0, input.moving.y))
            .normalize_or_zero();

        velocity.linvel = velocity.linvel.lerp(
            direction * speed,
            characteristics.acceleration * time.delta_seconds(),
        );

        if input.jumping && (state.standing || state.moving || state.crouching) {
            velocity.linvel.y += characteristics.jumping_high
        }
    }
}

fn update_state(
    time: Res<Time>,
    rapier: Res<RapierContext>,
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
        let gravity = 9.81 * gravity.0 * time.delta_seconds();

        velocity.linvel.y -= gravity;

        let collision = match rapier.cast_shape(
            transform.translation,
            transform.rotation,
            velocity.linvel,
            collider,
            4.0,
            true,
            QueryFilter::new().exclude_collider(entity),
        ) {
            Some((_, toi)) => match toi.details {
                Some(d) => {
                    let vector_to_collision = velocity.linvel * toi.toi;
                    let vector_to_surface = (vector_to_collision.project_onto(d.normal1)
                        + d.normal1)
                        .project_onto(velocity.linvel);

                    if vector_to_surface.angle_between(velocity.linvel) > 0.0 {
                        Some(d.normal1)
                    } else {
                        None
                    }
                }
                None => None,
            },
            None => None,
        };

        let mut grounded = false;
        if let Some(normal) = collision {
            velocity.linvel = velocity.linvel - velocity.linvel.project_onto(normal);

            if normal.angle_between(Vec3::Y) < consts::PI / 4.0 {
                grounded = true
            }
        };

        transform.translation += velocity.linvel;

        if grounded {
            let speed = velocity.linvel.xz().length();

            if state.standing && speed > 0.01 {
                state.standing = false;
                state.moving = true;
                continue;
            }

            if state.moving && speed < 0.01 {
                state.moving = false;
                state.standing = true;
                continue;
            }

            if state.rising && speed > 0.01 {
                state.rising = false;
                state.moving = true;
                continue;
            }

            if state.rising && speed < 0.01 {
                state.rising = false;
                state.standing = true;
                continue;
            }

            if state.falling && speed > 0.01 {
                state.falling = false;
                state.moving = true;
                continue;
            }

            if state.falling && speed < 0.01 {
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