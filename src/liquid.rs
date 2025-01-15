use std::ops::{Add, AddAssign};

use bevy::{
    app::Plugin,
    color::palettes::{
        css::{BLUE, RED},
        tailwind::{BLUE_200, BLUE_500},
    },
    ecs::component::Component,
    gizmos,
    log::tracing_subscriber::field::debug,
    math::VectorSpace,
    prelude::*,
    render::primitives::Aabb,
};
use bevy_rapier3d::{
    na::{Isometry, Isometry3, Quaternion, Vector},
    plugin::{RapierConfiguration, RapierContext},
    prelude::*,
};

use bevy_rapier3d::na::Unit;
use rand::Rng;

use crate::{
    library::{n_gon_area, n_gon_from_points},
    random::Random,
};

#[derive(Component, Reflect)]
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

fn projection_area(normal: Vec3, vertices: impl IntoIterator<Item = impl Into<Vec3>>) -> f32 {
    let normal = normal.normalize();

    let mut projection = Vec::new();

    let rotation = Quat::from_rotation_arc(normal, Vec3::Z);

    for vertex in vertices {
        let vertex: Vec3 = vertex.into();

        let point = rotation * vertex.reject_from(normal);

        projection.push(Vec2::new(point.x, point.y));
    }

    let aabb_projection_polygon = n_gon_from_points(projection);

    n_gon_area(&aabb_projection_polygon)
}

#[derive(Default)]
struct Force {
    linear: Vec3,
    angular: Vec3,
    center_of_mass: Vec3,
    point: Vec3,
    time: f32,
    mass: f32,
}

impl Force {
    pub fn with_point(mut self, point: Vec3) -> Self {
        self.point = point;
        self
    }

    pub fn with_center_of_mass(mut self, center_of_mass: Vec3) -> Self {
        self.center_of_mass = center_of_mass;
        self
    }

    pub fn with_time(mut self, time: f32) -> Self {
        self.time = time;
        self
    }

    pub fn with_mass(mut self, mass: f32) -> Self {
        self.mass = mass;
        self
    }

    pub fn velocity(&self) -> Velocity {
        return Velocity {
            linvel: (self.linear / self.mass) * self.time,
            angvel: (self.angular / self.mass) * self.time,
        };
    }

    pub fn impulse(&self) -> ExternalImpulse {
        ExternalImpulse {
            impulse: self.linear * self.time,
            torque_impulse: self.angular * self.time,
        }
    }
}

impl Add<Vec3> for Force {
    type Output = Self;

    fn add(self, force: Vec3) -> Self {
        Self {
            linear: self.linear + force,
            angular: self.angular + (self.point - self.center_of_mass).cross(force),
            center_of_mass: self.center_of_mass,
            point: self.point,
            time: self.time,
            mass: self.mass,
        }
    }
}

impl AddAssign<Vec3> for Force {
    fn add_assign(&mut self, force: Vec3) {
        self.linear += force;
        self.angular += (self.point - self.center_of_mass).cross(force);
    }
}

fn update(
    mut gizmos: Gizmos,
    rapier: Single<&RapierContext>,
    config: Single<&RapierConfiguration>,
    mut random: ResMut<Random>,
    mut time: Res<Time<Fixed>>,
    mut commands: Commands,
    mut collider_q: Query<(
        Entity,
        &Collider,
        &Floating,
        &GlobalTransform,
        &Velocity,
        Option<&VolumeScale>,
        Option<&ColliderMassProperties>,
    )>,
    liquid_q: Query<
        (
            &mut Liquid,
            &GlobalTransform,
            &Collider,
            Option<&ColliderMassProperties>,
        ),
        Without<Floating>,
    >,
) {
    for (entity, collider, floating, transform, velocity, volume_scale, mass_properties) in
        collider_q.iter_mut()
    {
        let (liquid, pool_transform, pool_collider, pool_mass_poperties) =
            liquid_q.get(floating.pool).unwrap();

        let pool_density = match mass_properties.unwrap_or(&ColliderMassProperties::default()) {
            ColliderMassProperties::Density(density) => *density,
            _ => 1000.0,
        };

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

        let mut points_inside = 0;

        let mut intersection_center = Vec3::ZERO;

        for _ in 0..liquid.sample_count {
            let point = Vec3 {
                x: random.rand.gen_range(mins.x..=maxs.x),
                y: random.rand.gen_range(mins.y..=maxs.y),
                z: random.rand.gen_range(mins.z..=maxs.z),
            };

            let mut inside_collider = false;
            let mut inside_pool = false;

            rapier.intersections_with_point(point, filter, |e| {
                inside_collider = inside_collider || e == entity;
                inside_pool = inside_pool || e == floating.pool;

                !inside_collider || !inside_pool
            });

            if inside_collider && inside_pool {
                points_inside += 1;

                if points_inside == 1 {
                    intersection_center = point;
                    continue;
                };

                intersection_center += point
            }
        }

        if points_inside == 0 {
            continue;
        }

        intersection_center /= points_inside as f32;

        let ratio = points_inside as f32 / liquid.sample_count as f32;

        let volume = aabb_volume * ratio * volume_scale.map(|s| s.scale).unwrap_or(1.0);

        let (mass, center_of_mass) =
            match mass_properties.unwrap_or(&ColliderMassProperties::default()) {
                ColliderMassProperties::Density(density) => {
                    (collider.raw.mass_properties(*density).mass(), Vec3::ZERO)
                }
                ColliderMassProperties::Mass(mass) => (*mass, Vec3::ZERO),
                ColliderMassProperties::MassProperties(properties) => {
                    (properties.mass, properties.local_center_of_mass)
                }
            };

        let gravity = config.gravity;

        let center_of_mass = transform.translation() + center_of_mass;

        let buoyant_force = -gravity.normalize() * (gravity.length() * pool_density * volume);

        let mut force = Force::default()
            .with_point(intersection_center)
            .with_center_of_mass(center_of_mass)
            .with_time(time.delta_secs())
            .with_mass(mass)
            + buoyant_force;

        let linear_velocity = force.velocity().linvel + velocity.linvel;
        if linear_velocity.length() > 0.0 {
            let aabb_projection_area =
                projection_area(linear_velocity, intersection_aabb.vertices());

            let area = aabb_projection_area * ratio;

            let drag_force = -linear_velocity.normalize()
                * (linear_velocity.dot(linear_velocity) * pool_density * area * 0.5);

            force += drag_force;
        }

        let angular_velocity = force.velocity().angvel + velocity.angvel;

        if angular_velocity.length() > 0.0 {
            let drag_force = -angular_velocity * (pool_density * volume).min(mass);
            force.angular += drag_force;
        }

        commands.entity(entity).insert(force.impulse());
    }
}
