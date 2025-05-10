use std::{f32::consts, time::Duration};

use bevy::{
    color::palettes::css::{ORANGE, RED},
    ecs::{component::ComponentId, entity, world::DeferredWorld},
    math::VectorSpace,
    pbr::NotShadowCaster,
    prelude::*,
    utils::HashMap,
};
use bevy_hanabi::{expr::TextureHandle, prelude::*};
use bevy_rapier3d::{
    plugin::RapierContext,
    prelude::{
        ActiveEvents, Collider, CollisionEvent, ExternalImpulse, MassProperties, QueryFilter,
        ReadMassProperties, RigidBody, Sensor,
    },
};

use crate::{
    billboard::BillboardMaterial,
    despawn::Despawn,
    library::{fibonacci_sphere, Spawnable},
};

#[derive(Resource, Default, Reflect, PartialEq, Clone)]
#[reflect(Resource)]
struct ExplosionAssets {
    plasm_effect: Handle<EffectAsset>,
    smoke_effect: Handle<EffectAsset>,
    smoke_texture: Handle<Image>,
    plasm_texture: Handle<Image>,
    explosion_sound: Handle<AudioSource>,
}

#[derive(Component, Reflect, Clone)]
#[component(on_add = explode)]
#[reflect(Component)]
pub struct Explosion {
    pub radius: f32,
    pub samples: usize,
    pub power: f32,
}

impl Explosion {
    pub fn new(radius: f32) -> Self {
        let samples = 200;

        Self {
            radius,
            samples,
            power: radius * 200.0,
        }
    }
}

impl Spawnable for Explosion {
    fn spawn<'a>(&self, commands: &'a mut Commands) -> EntityCommands<'a> {
        commands.spawn(self.clone())
    }
}

fn explode(mut world: DeferredWorld<'_>, entity: Entity, _: ComponentId) {
    let assets = world.get_resource::<ExplosionAssets>().unwrap().clone();

    world
        .commands()
        .entity(entity)
        .insert((
            Name::new("explosion"),
            AudioPlayer::new(assets.explosion_sound),
            Despawn::after(Duration::from_millis(5000)).recursive(),
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

pub struct ExplosionPlugin;

impl Plugin for ExplosionPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Explosion>()
            .register_type::<ExplosionAssets>()
            .add_systems(PreStartup, load)
            .add_systems(FixedPreUpdate, update);
    }
}

fn create_smoke_effect() -> EffectAsset {
    let mut smoke_size_gradient = Gradient::new();
    smoke_size_gradient.add_key(0.0, Vec3::ZERO);
    smoke_size_gradient.add_key(0.01, Vec3::splat(2.0));
    smoke_size_gradient.add_key(1.0, Vec3::splat(4.0));

    let mut smoke_gradient = Gradient::new();
    smoke_gradient.add_key(0.0, Vec4::new(0., 0., 0., 1.));
    smoke_gradient.add_key(0.5, Vec4::new(0., 0., 0., 1.));
    smoke_gradient.add_key(1.0, Vec4::splat(0.));

    let writer = ExprWriter::new();

    let texture_slot = writer.lit(0u32).expr();
    let lifetime = writer.lit(5.0).expr();
    let rotation = (writer.rand(ScalarType::Float) * writer.lit(std::f32::consts::TAU)).expr();
    let size = (writer.rand(ScalarType::Float)).expr();
    let drag = writer.lit(3.).expr();
    let accel = writer.lit(Vec3::Y * 0.2).expr();
    let center = writer.lit(Vec3::ZERO).expr();
    let speed = writer.lit(4.).expr();
    let radius = writer.lit(0.01).expr();

    let mut module = writer.finish();

    module.add_texture_slot("smoke");

    EffectAsset::new(8, SpawnerSettings::once(8.0.into()), module)
        .with_name("smoke")
        .render(OrientModifier::new(OrientMode::FaceCameraPosition))
        .init(SetAttributeModifier::new(Attribute::SIZE, size))
        .init(SetPositionSphereModifier {
            center,
            radius,
            dimension: ShapeDimension::Volume,
        })
        .render(SizeOverLifetimeModifier {
            gradient: smoke_size_gradient,
            screen_space_size: false,
        })
        .init(SetVelocitySphereModifier { center, speed })
        .init(SetAttributeModifier::new(Attribute::LIFETIME, lifetime))
        .init(SetAttributeModifier::new(Attribute::F32_0, rotation))
        .update(LinearDragModifier::new(drag))
        .update(AccelModifier::new(accel))
        .render(ParticleTextureModifier {
            texture_slot,
            sample_mapping: ImageSampleMapping::ModulateOpacityFromR,
        })
        .render(ColorOverLifetimeModifier::new(smoke_gradient))
}

fn create_plasm_effect() -> EffectAsset {
    let mut size_gradient = Gradient::new();
    size_gradient.add_key(0.0, Vec3::ZERO);
    size_gradient.add_key(1.0, Vec3::splat(3.0));

    let mut color_gradient = Gradient::new();
    color_gradient.add_key(0.0, Vec4::new(1., 1., 1., 0.5));
    color_gradient.add_key(0.5, Vec4::new(1., 1., 1., 0.5));
    color_gradient.add_key(1.0, Vec4::new(1., 1., 1., 0.));

    let writer = ExprWriter::new();

    let texture_slot = writer.lit(0u32).expr();
    let lifetime = writer.lit(0.2).expr();
    let size = (writer.rand(ScalarType::Float)).expr();
    let center = writer.lit(Vec3::ZERO).expr();
    let radius = writer.lit(0.5).expr();

    let mut module = writer.finish();

    module.add_texture_slot("plasm");

    EffectAsset::new(8, SpawnerSettings::once(8.0.into()), module)
        .with_name("plasm")
        .render(OrientModifier::new(OrientMode::FaceCameraPosition))
        .init(SetAttributeModifier::new(Attribute::SIZE, size))
        .init(SetPositionSphereModifier {
            center,
            radius,
            dimension: ShapeDimension::Volume,
        })
        .render(SizeOverLifetimeModifier {
            gradient: size_gradient,
            screen_space_size: false,
        })
        .render(ColorOverLifetimeModifier::new(color_gradient))
        .render(ParticleTextureModifier {
            texture_slot,
            sample_mapping: ImageSampleMapping::Modulate,
        })
        .init(SetAttributeModifier::new(Attribute::LIFETIME, lifetime))
}

fn load(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut effects: ResMut<Assets<EffectAsset>>,
) {
    let assets = ExplosionAssets {
        plasm_effect: effects.add(create_plasm_effect()),
        smoke_effect: effects.add(create_smoke_effect()),
        plasm_texture: asset_server.load("explosion/plasm.png"),
        smoke_texture: asset_server.load("explosion/smoke.png"),
        explosion_sound: asset_server.load("explosion/sound.ogg"),
    };

    commands.insert_resource(assets.clone());

    commands.spawn((
        EffectMaterial {
            images: vec![assets.smoke_texture],
        },
        ParticleEffect::new(assets.smoke_effect),
        Despawn::after(Duration::from_millis(8)),
    ));
    commands.spawn((
        EffectMaterial {
            images: vec![assets.plasm_texture],
        },
        ParticleEffect::new(assets.plasm_effect),
        Despawn::after(Duration::from_millis(8)),
    ));
}

fn update(
    mut commands: Commands,
    explosion_q: Query<(Entity, &GlobalTransform, &Explosion)>,
    rapier: Single<&RapierContext>,
    body_q: Query<(
        &RigidBody,
        &GlobalTransform,
        Option<&ExternalImpulse>,
        Option<&ReadMassProperties>,
    )>,
) {
    for (entity, transform, explosion) in explosion_q.iter() {
        commands.entity(entity).remove::<Explosion>();

        let mut bodies = HashMap::<Entity, ExternalImpulse>::new();

        for direction in fibonacci_sphere(explosion.samples) {
            let Some((collider, intersection)) = rapier.cast_ray_and_get_normal(
                transform.translation(),
                direction * explosion.radius,
                1.0,
                true,
                QueryFilter::default(),
            ) else {
                continue;
            };

            let body = rapier.collider_parent(collider).unwrap_or(collider);

            let (rigid_body, transform, impulse, mass_properties) = body_q.get(body).unwrap();

            if *rigid_body != RigidBody::Dynamic {
                continue;
            }

            let center_of_mass = mass_properties
                .map(|properties| properties.local_center_of_mass)
                .unwrap_or_default();

            let force: f32 = 1.0 - intersection.time_of_impact;

            let impulse = bodies.get(&body).or(impulse).cloned().unwrap_or_default()
                + ExternalImpulse::at_point(
                    direction * (explosion.power * force / explosion.samples as f32),
                    intersection.point,
                    transform.translation() + center_of_mass,
                );

            bodies.insert(body, impulse);
        }

        for (body, impulse) in bodies {
            commands.entity(body).insert(impulse);
        }
    }
}
