use bevy::{app::Plugin, ecs::component::Component, prelude::*};
use bevy_rapier3d::{
    plugin::{RapierConfiguration, RapierContext},
    prelude::{CollisionEvent, ExternalImpulse, Velocity},
};

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Liquid {
    viscosity: f32,
}

impl Default for Liquid {
    fn default() -> Self {
        Self { viscosity: 0.5 }
    }
}

impl Liquid {
    pub fn new(viscosity: f32) -> Self {
        Self {
            viscosity: viscosity,
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Floating {
    pool: Entity,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Buoyant {
    force: f32,
}

impl Default for Buoyant {
    fn default() -> Self {
        Self { force: 1.0 }
    }
}

impl Buoyant {
    pub fn new(force: f32) -> Self {
        Self { force: force }
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
            .register_type::<Buoyant>()
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
    rapier: Single<&RapierContext>,
    config: Single<&RapierConfiguration>,
    mut commands: Commands,
    mut collider_q: Query<(
        Entity,
        &Floating,
        &GlobalTransform,
        &Velocity,
        Option<&Buoyant>,
    )>,
    liquid_q: Query<&mut Liquid, Without<Floating>>,
) {
    for (entity, floating, transform, velocity, buoyant) in collider_q.iter_mut() {
        let Some(contact) = rapier.contact_pair(floating.pool, entity) else {
            continue;
        };

        let manifold = contact.manifold(0).unwrap();

        let points_len = manifold.num_points() as f32;

        let mut external_impulse = ExternalImpulse::default();

        let position = transform.translation();

        let liquid = liquid_q.get(floating.pool).unwrap();

        let buoyant_force = -config.gravity.normalize() * buoyant.map(|b| b.force).unwrap_or(1.0);

        for point in manifold.points() {
            let distance = point.dist().abs();

            let linear_resistance = -velocity.linvel * (liquid.viscosity * distance).min(1.0);
            let angular_resistance = -velocity.angvel * (liquid.viscosity * distance).min(1.0);

            let impulse = buoyant_force * distance + linear_resistance;

            let point_position = position + (-transform.rotation() * point.local_p2());

            external_impulse +=
                ExternalImpulse::at_point(impulse / points_len, point_position, position);

            external_impulse.torque_impulse += angular_resistance;
        }

        commands.entity(entity).insert(external_impulse);
    }
}
