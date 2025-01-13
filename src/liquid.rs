use bevy::{
    app::Plugin,
    color::palettes::{
        css::{BLUE, RED},
        tailwind::{BLUE_200, BLUE_500},
    },
    ecs::component::Component,
    gizmos,
    log::tracing_subscriber::field::debug,
    prelude::*,
    render::primitives::Aabb,
};
use bevy_rapier3d::{
    na::{Isometry, Isometry3, Quaternion, Vector},
    plugin::{RapierConfiguration, RapierContext},
    prelude::*,
};

use bevy_rapier3d::na::Unit;

use crate::random::Random;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Liquid {
    density: f32,
}

impl Default for Liquid {
    fn default() -> Self {
        Self { density: 0.5 }
    }
}

impl Liquid {
    pub fn new(density: f32) -> Self {
        Self { density }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Floating {
    pool: Entity,
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
            .add_systems(FixedFirst, resolve.in_set(LiquidSystems::Resolve))
            .add_systems(FixedPreUpdate, update.in_set(LiquidSystems::Update));
    }
}

fn resolve(
    rapier: Single<&RapierContext>,
    mut commands: Commands,
    mut collisions: EventReader<CollisionEvent>,
    liquid_q: Query<&Liquid>,
) {
    for event in collisions.read() {
        match event {
            CollisionEvent::Started(sensor_entity, collider_entity, _) => {
                if !liquid_q.contains(*sensor_entity) {
                    continue;
                };

                if rapier.collider_parent(*collider_entity).is_none() {
                    continue;
                }

                commands.entity(*collider_entity).insert(Floating {
                    pool: *sensor_entity,
                });
            }
            CollisionEvent::Stopped(sensor_entity, collider_entity, _) => {
                if !liquid_q.contains(*sensor_entity) {
                    continue;
                };

                commands.entity(*collider_entity).remove::<Floating>();
            }
        };
    }
}

fn update(
    mut gizmos: Gizmos,
    rapier: Single<&RapierContext>,
    config: Single<&RapierConfiguration>,
    mut commands: Commands,
    mut collider_q: Query<(Entity, &Collider, &Floating, &GlobalTransform, &Velocity)>,
    liquid_q: Query<(&mut Liquid, &GlobalTransform, &Collider), Without<Floating>>,
) {
    for (entity, collider, floating, transform, velocity) in collider_q.iter_mut() {
        let Some(contact) = rapier.contact_pair(floating.pool, entity) else {
            continue;
        };

        let manifold = contact.manifold(0).unwrap();

        let points_len = manifold.num_points() as f32;

        let mut external_impulse = ExternalImpulse::default();

        let collider_position = transform.translation();

        let (liquid, pool_transform, pool_collider) = liquid_q.get(floating.pool).unwrap();

        let pool_position = pool_transform.translation();

        let buoyant_force = -config.gravity * liquid.density;

        manifold.point(0).unwrap().local_p1();

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

        let grid_resolution = 5.0;

        let min_x = (intersection_aabb.mins.x * grid_resolution) as i32 + 1;
        let max_x = (intersection_aabb.maxs.x * grid_resolution) as i32 + 1;

        let min_y = (intersection_aabb.mins.y * grid_resolution) as i32 + 1;
        let max_y = (intersection_aabb.maxs.y * grid_resolution) as i32 + 1;

        let min_z = (intersection_aabb.mins.z * grid_resolution) as i32 + 1;
        let max_z = (intersection_aabb.maxs.z * grid_resolution) as i32 + 1;

        let filter = QueryFilter::default();

        for x in min_x..max_x {
            for y in min_y..max_y {
                for z in min_z..max_z {
                    let position = Vec3::new(
                        (x as f32) / grid_resolution,
                        (y as f32) / grid_resolution,
                        (z as f32) / grid_resolution,
                    );

                    let point = Transform::from_translation(position)
                        .with_scale(Vec3::ONE / grid_resolution);

                    let mut inside_collider = false;
                    let mut inside_pool = false;

                    rapier.intersections_with_point(position, filter, |e| {
                        inside_collider = inside_collider || e == entity;
                        inside_pool = inside_pool || e == floating.pool;

                        !inside_collider || !inside_pool
                    });

                    if !inside_collider || !inside_pool {
                        continue;
                    }

                    external_impulse += ExternalImpulse::at_point(
                        buoyant_force / grid_resolution.powi(3),
                        position,
                        collider_position,
                    );

                    #[cfg(debug_assertions)]
                    gizmos.cuboid(point, BLUE_500);
                }
            }
        }

        commands.entity(entity).insert(external_impulse);
    }
}
