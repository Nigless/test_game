use std::time::Duration;

use bevy::{
    color::palettes::{
        basic,
        css::{ORANGE, RED},
    },
    ecs::{component::ComponentId, world::DeferredWorld},
    prelude::*,
    transform,
    utils::HashMap,
};
use bevy_hanabi::prelude::*;
use bevy_rapier3d::{
    plugin::RapierContext,
    prelude::{ExternalImpulse, QueryFilter, ReadMassProperties, RigidBody},
};

use crate::{
    library::fibonacci_sphere,
    plugins::{despawn::Despawn, health::DamageInvokedEvent, settings::Settings},
};

mod startup;
use startup::startup;

#[derive(Resource, Clone)]
pub(self) struct ExplosionAssets {
    plasm_effect: Handle<EffectAsset>,
    smoke_effect: Handle<EffectAsset>,
    smoke_texture: Handle<Image>,
    plasm_texture: Handle<Image>,
    explosion_sound: Handle<AudioSource>,
}

#[derive(Component, Default)]
pub struct ExplosionHits {
    pub rays: Vec<(Vec3, Vec3)>,
}

#[derive(Component, Reflect, Clone)]
#[component(on_add = resolve)]
#[reflect(Component)]
pub struct Explosion {
    pub radius: f32,
    pub samples: usize,
    pub power: f32,
    pub damage: u16,
}

fn resolve(mut world: DeferredWorld<'_>, entity: Entity, _: ComponentId) {
    let assets = world.get_resource::<ExplosionAssets>().cloned().unwrap();

    world
        .commands()
        .entity(entity)
        .insert_if_new((
            Name::new("explosion"),
            AudioPlayer::new(assets.explosion_sound),
            Despawn::after(Duration::from_millis(5000)).recursive(),
            Transform::default(),
        ))
        .with_children(|commands| {
            commands.spawn((
                EffectMaterial {
                    images: vec![assets.smoke_texture],
                },
                ParticleEffect::new(assets.smoke_effect),
                Transform::default(),
            ));
            commands.spawn((
                EffectMaterial {
                    images: vec![assets.plasm_texture],
                },
                ParticleEffect::new(assets.plasm_effect),
                Transform::default(),
            ));
            commands.spawn((
                PointLight {
                    intensity: 10_000_000.0,
                    color: ORANGE.into(),
                    shadows_enabled: true,
                    ..default()
                },
                Transform::default(),
                Despawn::after(Duration::from_millis(100)).recursive(),
            ));
        });
}

impl Explosion {
    pub fn new(radius: f32) -> Self {
        let samples = 1000;

        Self {
            radius,
            samples,
            power: radius * 100.0,
            damage: 50,
        }
    }
}

pub struct ExplosionPlugin;

impl Plugin for ExplosionPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Explosion>()
            .add_systems(FixedPreUpdate, explode)
            .add_systems(PreStartup, startup)
            .add_systems(
                PreUpdate,
                (debug_explosion, debug_explosion_hit)
                    .run_if(Settings::is(|s| s.dev_settings.show_explosion_hits)),
            );
    }
}

fn debug_explosion(mut gismos: Gizmos, entity_q: Query<(&GlobalTransform, &Explosion)>) {
    for (transform, explosion) in entity_q.iter() {
        gismos.sphere(
            Isometry3d::from_translation(transform.translation()),
            explosion.radius,
            basic::RED,
        );

        gismos.sphere(
            Isometry3d::from_translation(transform.translation()),
            0.1,
            basic::YELLOW,
        );
    }
}

fn debug_explosion_hit(mut gismos: Gizmos, entity_q: Query<&ExplosionHits>) {
    for hits in entity_q.iter() {
        for (position, direction) in &hits.rays {
            let length = 0.2;

            let start = position - direction * (length / 2.0);

            gismos.ray(start, direction * length, basic::RED);
        }
    }
}

fn explode(
    mut commands: Commands,
    explosion_q: Query<(&Transform, &Explosion), Added<Explosion>>,
    rapier: Single<&RapierContext>,
    settings: Res<Settings>,
    body_q: Query<(
        &RigidBody,
        &GlobalTransform,
        Option<&ExternalImpulse>,
        Option<&ReadMassProperties>,
    )>,
) {
    for (transform, explosion) in explosion_q.iter() {
        let mut bodies = HashMap::<Entity, ExternalImpulse>::new();

        let mut hits = ExplosionHits::default();

        for direction in fibonacci_sphere(explosion.samples) {
            let Some((collider, intersection)) = rapier.cast_ray_and_get_normal(
                transform.translation,
                direction * explosion.radius,
                1.0,
                true,
                QueryFilter::default().exclude_sensors(),
            ) else {
                continue;
            };

            if settings.dev_settings.show_explosion_hits {
                hits.rays.push((intersection.point, direction));
            }

            let body = rapier.collider_parent(collider).unwrap_or(collider);

            let Ok((rigid_body, transform, impulse, mass_properties)) = body_q.get(body) else {
                continue;
            };

            if *rigid_body != RigidBody::Dynamic {
                continue;
            }

            let center_of_mass = mass_properties
                .map(|properties| properties.local_center_of_mass)
                .unwrap_or_default();

            let mut impulse = bodies.get(&body).or(impulse).cloned().unwrap_or_default();

            if intersection.time_of_impact == 0.0 {
                let direction = transform.translation() - intersection.point;

                bodies.insert(
                    body,
                    impulse
                        + ExternalImpulse::at_point(
                            direction.normalize() * explosion.power,
                            intersection.point,
                            transform.translation() + center_of_mass,
                        ),
                );

                break;
            }

            impulse += ExternalImpulse::at_point(
                direction * (explosion.power / explosion.samples as f32),
                intersection.point,
                transform.translation() + center_of_mass,
            );

            bodies.insert(body, impulse);
        }

        if settings.dev_settings.show_explosion_hits {
            commands.spawn((hits, Despawn::after(Duration::from_millis(5000))));
        }

        for (body, impulse) in bodies {
            commands.entity(body).try_insert(impulse);
            commands.entity(body).trigger(DamageInvokedEvent(
                (impulse.impulse.length() * explosion.damage as f32) as u16,
            ));
        }
    }
}
