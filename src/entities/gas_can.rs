use bevy::{
    ecs::{component::ComponentId, world::DeferredWorld},
    prelude::*,
};
use bevy_rapier3d::prelude::{RigidBody, Velocity};

use crate::{
    entities::explosion::Explosion,
    plugins::{
        health::{DeadEvent, Health},
        prefab::{Prefab, PrefabCollection},
        serializable::Serializable,
    },
};

#[derive(Component, Reflect)]
#[reflect(Component)]
#[component(on_add = spawn)]
pub struct GasCan;

fn spawn(mut world: DeferredWorld<'_>, entity: Entity, _: ComponentId) {
    world
        .commands()
        .entity(entity)
        .insert_if_new((
            Name::new("gas_can"),
            Health::new(50),
            RigidBody::Dynamic,
            Transform::default(),
            Velocity::default(),
        ))
        .insert((
            Prefab::new("gas_can/model.glb"),
            Serializable::default()
                .with::<GasCan>()
                .with::<Transform>()
                .with::<Velocity>(),
        ));
}

pub struct GasCanPlugin;

impl Plugin for GasCanPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<GasCan>()
            .add_systems(PreStartup, startup)
            .add_observer(handle_death);
    }
}

fn startup(mut prefabs: ResMut<PrefabCollection>, server: Res<AssetServer>) {
    prefabs.insert(
        "gas_can/model.glb",
        server.load(GltfAssetLabel::Scene(0).from_asset("gas_can/model.glb")),
    );
}

fn handle_death(
    trigger: Trigger<DeadEvent>,
    mut commands: Commands,
    entity_q: Query<(&Transform, Option<&Parent>), With<GasCan>>,
) {
    let entity = trigger.entity();

    let Ok((transform, parent)) = entity_q.get(entity) else {
        return;
    };

    commands.entity(entity).despawn_recursive();

    let explosion = commands
        .spawn((Explosion::new(3.0), transform.clone()))
        .id();

    if let Some(parent) = parent {
        commands.entity(explosion).set_parent(parent.get());
    }
}
