use std::{
    default,
    f32::consts,
    ops::{Add, AddAssign, Index},
};

use bevy::{
    app::Plugin,
    color::palettes::{
        css::{BLUE, RED},
        tailwind::{BLUE_200, BLUE_500},
    },
    ecs::component::{Component, ComponentHooks, StorageType},
    gizmos,
    log::tracing_subscriber::field::debug,
    math::VectorSpace,
    prelude::*,
    render::primitives::Aabb,
    scene::ron::extensions,
    transform,
};
use bevy_hanabi::velocity;
use bevy_rapier3d::{
    na::{Isometry, Isometry3, Quaternion, Vector},
    plugin::{RapierConfiguration, RapierContext},
    prelude::*,
    rapier::prelude::RigidBodyType,
};

use bevy_rapier3d::na::Unit;
use rand::Rng;
use rand_chacha::rand_core::le;
use serde::{Deserialize, Serialize};

use crate::{
    library::{n_gon_area, n_gon_from_points, project_points},
    random::Random,
};

#[derive(Reflect)]
struct Intersection {
    points: Vec<Vec3>,
    center: Vec3,
    volume: f32,
}

#[derive(Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Liquid {
    density: f32,
    sample_count: i32,
}

impl Default for Liquid {
    fn default() -> Self {
        Self {
            density: 0.5,
            sample_count: 100,
        }
    }
}

impl Component for Liquid {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_add(|mut world, entity, _component_id| {
            world
                .commands()
                .entity(entity)
                .insert(Sensor)
                .insert(ActiveEvents::COLLISION_EVENTS);
        });
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct VolumeScale {
    scale: f32,
}

impl Default for VolumeScale {
    fn default() -> Self {
        Self { scale: 1.0 }
    }
}

impl VolumeScale {
    pub fn new(scale: f32) -> Self {
        Self { scale }
    }
}

impl Liquid {
    pub fn new(density: f32) -> Self {
        Self {
            density,
            ..default()
        }
    }

    pub fn with_sample_count(mut self, count: i32) -> Self {
        self.sample_count = count;
        self
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Floating {
    pool: Entity,
    intersection: Option<Intersection>,
}

impl Floating {
    pub fn new(entity: Entity) -> Self {
        Self {
            pool: entity,
            intersection: default(),
        }
    }

    pub fn pool(&self) -> Entity {
        self.pool
    }
}

#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub enum LiquidSystems {
    Resolve,
    Update,
}

pub struct LiquidPlugin;

impl Plugin for LiquidPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Liquid>()
            .register_type::<Floating>()
            .register_type::<VolumeScale>()
            .add_systems(FixedFirst, resolve.in_set(LiquidSystems::Resolve))
            .add_systems(FixedPreUpdate, update.in_set(LiquidSystems::Update));
    }
}

fn resolve(
    rapier: Single<&RapierContext>,
    mut commands: Commands,
    mut collisions: EventReader<CollisionEvent>,
    liquid_q: Query<(Entity, &Liquid)>,
) {
    for event in collisions.read() {
        match event {
            CollisionEvent::Started(entity_a, entity_b, _) => {
                let (sensor, collider) = if liquid_q.contains(*entity_a) {
                    (entity_a, entity_b)
                } else if liquid_q.contains(*entity_b) {
                    (entity_b, entity_a)
                } else {
                    continue;
                };

                if rapier.collider_parent(*collider).is_none() {
                    continue;
                }

                commands.entity(*collider).insert(Floating::new(*sensor));
            }
            CollisionEvent::Stopped(entity_a, entity_b, _) => {
                let collider = if liquid_q.contains(*entity_a) {
                    entity_b
                } else if liquid_q.contains(*entity_b) {
                    entity_a
                } else {
                    continue;
                };

                if rapier
                    .intersection_pair(*entity_a, *entity_b)
                    .unwrap_or(false)
                {
                    continue;
                }

                commands.entity(*collider).remove::<Floating>();
            }
        };
    }
}

fn update(
    mut random: ResMut<Random>,
    rapier: Single<&mut RapierContext>,
    mut collider_q: Query<(
        Entity,
        &GlobalTransform,
        &Collider,
        &mut Floating,
        Option<&VolumeScale>,
    )>,
    liquid_q: Query<(&mut Liquid, &GlobalTransform, &Collider), Without<Floating>>,
) {
    for (collider_entity, transform, collider, mut floating, volume_scale) in collider_q.iter_mut()
    {
        let (liquid, pool_transform, pool_collider) = liquid_q.get(floating.pool).unwrap();

        let collider_aabb = collider.raw.compute_aabb(&Isometry3::from_parts(
            transform.translation().into(),
            transform.rotation().into(),
        ));

        let pool_aabb = pool_collider.raw.compute_aabb(&Isometry3::from_parts(
            pool_transform.translation().into(),
            pool_transform.rotation().into(),
        ));

        let Some(intersection_aabb) = pool_aabb.intersection(&collider_aabb) else {
            continue;
        };

        let aabb_volume = intersection_aabb.volume();

        if aabb_volume.is_nan() {
            continue;
        }

        let maxs = intersection_aabb.maxs;
        let mins = intersection_aabb.mins;

        let filter = QueryFilter::default();

        let mut points_inside = Vec::<Vec3>::default();
        let mut intersection_center = Vec3::ZERO;

        for _ in 0..liquid.sample_count {
            let point = Vec3 {
                x: random.rand.gen_range(mins.x..=maxs.x),
                y: random.rand.gen_range(mins.y..=maxs.y),
                z: random.rand.gen_range(mins.z..=maxs.z),
            };

            let mut inside_collider = false;
            let mut inside_pool = false;

            rapier.intersections_with_point(point, filter, |intersection| {
                inside_collider = inside_collider || intersection == collider_entity;
                inside_pool = inside_pool || intersection == floating.pool;

                true
            });

            if inside_collider && inside_pool {
                points_inside.push(point);
                intersection_center += point
            }
        }

        if points_inside.is_empty() {
            floating.intersection = None;
            continue;
        }

        let ratio = points_inside.len() as f32 / liquid.sample_count as f32;

        floating.intersection = Some(Intersection {
            center: intersection_center / points_inside.len() as f32,
            points: points_inside,
            volume: aabb_volume * ratio * volume_scale.unwrap_or(&default()).scale,
        });
    }
}

fn apply_buoyant_force(
    mut commands: Commands,
    time: Res<Time<Fixed>>,
    liquid_q: Query<&Liquid>,
    collider_q: Query<(Entity, &Floating)>,
    body_q: Query<(Entity, &ReadMassProperties, &GlobalTransform)>,
    config: Single<&RapierConfiguration>,
    rapier: Single<&RapierContext>,
) {
    for (entity, floating) in collider_q.iter() {
        let gravity = config.gravity;

        let Ok(liquid) = liquid_q.get(floating.pool()) else {
            continue;
        };

        let Some(intersection) = &floating.intersection else {
            continue;
        };

        let Some((body, mass_props, transform)) = rapier
            .collider_parent(entity)
            .and_then(|e| body_q.get(e).ok())
        else {
            continue;
        };

        let buoyant_force =
            -gravity.normalize() * (gravity.length() * liquid.density * intersection.volume);

        commands.entity(body).insert(ExternalImpulse::at_point(
            buoyant_force * time.delta_secs(),
            intersection.center,
            transform.translation() + mass_props.local_center_of_mass,
        ));
    }
}

fn apply_drag_force(
    mut commands: Commands,
    time: Res<Time<Fixed>>,
    liquid_q: Query<&Liquid>,
    collider_q: Query<(Entity, &Floating, Option<&VolumeScale>)>,
    body_q: Query<(Entity, &ReadMassProperties, &GlobalTransform, &Velocity)>,
    rapier: Single<&RapierContext>,
) {
    for (entity, floating, volume_scale) in collider_q.iter() {
        let scale = volume_scale.unwrap_or(&default()).scale;

        let Ok(liquid) = liquid_q.get(floating.pool()) else {
            continue;
        };

        let Some(intersection) = &floating.intersection else {
            continue;
        };

        let Some((body, mass_props, transform, velocity)) = rapier
            .collider_parent(entity)
            .and_then(|e| body_q.get(e).ok())
        else {
            continue;
        };

        let area = n_gon_area(n_gon_from_points(project_points(
            intersection.points.clone(),
            velocity.linvel,
        ))) * scale;

        let drag_force = 0.5 * liquid.density * velocity.linvel.length_squared() * 1.0 * area;

        commands.entity(body).insert(ExternalImpulse::at_point(
            -velocity.linvel.normalize() * drag_force * time.delta_secs(),
            intersection.center,
            transform.translation() + mass_props.local_center_of_mass,
        ));
    }
}
