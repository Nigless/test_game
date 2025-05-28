use bevy::{
    core::Name,
    ecs::{component::ComponentId, world::DeferredWorld},
    reflect::Reflect,
    transform::components::Transform,
};
use bevy_rapier3d::prelude::{ColliderMassProperties, RigidBody, Velocity};

use crate::{
    library::Spawnable,
    prefab::{Prefab, PrefabCollection},
    saves::Serializable,
};

use bevy::{ecs::component::Component, prelude::*};

#[derive(Component, Reflect, Clone)]
#[require(Transform, Velocity)]
#[component(on_add = spawn)]
#[reflect(Component)]
pub struct TrafficCone;

fn spawn(mut world: DeferredWorld<'_>, entity: Entity, _: ComponentId) {
    world.commands().entity(entity).insert_if_new((
        Name::new("traffic_cone"),
        Prefab::new("traffic_cone/model.glb"),
        Serializable::default()
            .with::<TrafficCone>()
            .with::<Transform>()
            .with::<Velocity>(),
        RigidBody::Dynamic,
        ColliderMassProperties::Mass(3.0),
    ));
}

impl Spawnable for TrafficCone {
    fn spawn<'a>(&self, commands: &'a mut Commands) -> EntityCommands<'a> {
        commands.spawn(self.clone())
    }
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
