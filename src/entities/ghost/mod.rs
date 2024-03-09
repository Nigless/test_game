use std::f32::consts::PI;

use crate::control::Controls;
use crate::utils::collider_bundle::ColliderBundle;
use bevy::asset::VisitAssetDependencies;
use bevy::render::render_resource::encase::vector;
use bevy::scene::ron::value;
use bevy::ui::update;
use bevy::{ecs::reflect, input::mouse::MouseMotion};
use bevy_rapier3d::control::{
    self, CharacterLength, KinematicCharacterController, KinematicCharacterControllerOutput,
};
use bevy_rapier3d::dynamics::{GravityScale, RigidBody, Velocity};
use bevy_rapier3d::na::Isometry;
use bevy_rapier3d::pipeline::QueryFilter;
use bevy_rapier3d::plugin::RapierContext;
use bevy_rapier3d::prelude::Collider;

use bevy::prelude::*;
use bevy_rapier3d::prelude::LockedAxes;
use bevy_rapier3d::rapier::geometry::ColliderSet;
use bevy_rapier3d::rapier::pipeline::QueryPipeline;

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
        app.register_type::<State>()
            .register_type::<Characteristics>()
            .add_systems(
                PreUpdate,
                (
                    resolve.before(look_around),
                    look_around.before(walk),
                    walk.before(apply_physics),
                    apply_physics,
                ),
            );
    }
}

fn resolve(entity_q: Query<Entity, With<Unresolved>>, mut commands: Commands) {
    for entity in entity_q.iter() {
        let camera = commands
            .spawn(Camera3dBundle {
                transform: Transform::from_xyz(0.0, 1.8 - 0.2, 0.0),
                ..default()
            })
            .id();

        commands
            .entity(entity)
            .remove::<Unresolved>()
            .add_child(camera);
    }
}

fn look_around(
    mut mouse: EventReader<MouseMotion>,
    mut character_q: Query<(&mut Transform, &State), Without<Unresolved>>,
) {
    let mut rotation = Vec2::ZERO;

    for event in mouse.read().into_iter() {
        rotation += Vec2::new(event.delta.x, event.delta.y);
    }

    for (mut transform, state) in character_q.iter_mut() {
        transform.rotation *= Quat::from_rotation_y(-rotation.x * 0.002);
    }
}

fn walk(
    time: Res<Time>,
    controls: Res<Controls>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut character_q: Query<(&mut Velocity, &mut Transform, &State, &Characteristics)>,
) {
    let mut direction = Vec2::ZERO;

    if keyboard.pressed(controls.move_left) {
        direction += Vec2::new(-1.0, 0.0);
    }

    if keyboard.pressed(controls.move_right) {
        direction += Vec2::new(1.0, 0.0);
    }

    if keyboard.pressed(controls.move_forward) {
        direction += Vec2::new(0.0, -1.0);
    }

    if keyboard.pressed(controls.move_backward) {
        direction += Vec2::new(0.0, 1.0);
    }

    let jumping = keyboard.just_pressed(controls.jump);
    let running = keyboard.pressed(controls.run);

    for (mut velocity, mut transform, state, characteristics) in character_q.iter_mut() {
        let speed = if state.standing || state.moving {
            if running {
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
            .mul_vec3(Vec3::new(direction.x, 0.0, direction.y))
            .normalize_or_zero();

        velocity.linvel = velocity.linvel.lerp(
            direction * speed,
            characteristics.acceleration * time.delta_seconds(),
        );

        if jumping && (state.standing || state.moving || state.crouching) {
            velocity.linvel.y += characteristics.jumping_high
        }
    }
}

fn apply_physics(
    time: Res<Time>,
    rapier: Res<RapierContext>,
    mut character_q: Query<(
        Entity,
        &mut State,
        &mut Velocity,
        &Collider,
        &mut Transform,
        &GravityScale,
    )>,
) {
    for (entity, mut state, mut velocity, collider, mut transform, gravity) in
        character_q.iter_mut()
    {
        let gravity = -9.81 * gravity.0;
        velocity.linvel.y = velocity.linvel.y.lerp(gravity, time.delta_seconds());

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

            if normal.angle_between(Vec3::Y) < PI / 4.0 {
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
