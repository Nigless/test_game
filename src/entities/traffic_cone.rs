use bevy::{
    core::Name,
    ecs::{component::ComponentId, world::DeferredWorld},
    reflect::Reflect,
    transform::components::Transform,
};
use bevy_rapier3d::prelude::{ColliderMassProperties, MassProperties, RigidBody, Velocity};

use crate::plugins::{
    prefab::{Prefab, PrefabCollection},
    serializable::Serializable,
};

use bevy::{ecs::component::Component, prelude::*};

#[derive(Component, Reflect, Clone)]
#[component(on_add = spawn)]
#[reflect(Component)]
pub struct TrafficCone;

fn spawn(mut world: DeferredWorld<'_>, entity: Entity, _: ComponentId) {
    world
        .commands()
        .entity(entity)
        .insert_if_new((
            Name::new("traffic_cone"),
            Prefab::new("traffic_cone/model.glb"),
            RigidBody::Dynamic,
            Transform::default(),
            Velocity::default(),
            ColliderMassProperties::Mass(3.0),
        ))
        .insert((Serializable::default()
            .with::<TrafficCone>()
            .with::<Transform>()
            .with::<Velocity>(),));
}

pub struct TrafficConePlugin;

impl Plugin for TrafficConePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<TrafficCone>()
            .add_systems(PreStartup, startup);
    }
}

fn startup(mut prefabs: ResMut<PrefabCollection>, server: Res<AssetServer>) {
    prefabs.insert(
        "traffic_cone/model.glb",
        server.load(GltfAssetLabel::Scene(0).from_asset("traffic_cone/model.glb")),
    );
}
