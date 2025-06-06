use std::{f32::consts, time::Duration};

use bevy::prelude::*;
use bevy_hanabi::prelude::*;

use super::ExplosionAssets;

use crate::{plugins::billboard::BillboardMaterial, plugins::despawn::Despawn};

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

pub fn startup(
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
