mod test_scene;
use std::f32::consts;

use bevy::prelude::*;
use bevy_rapier3d::prelude::RigidBody;
use test_scene::create_test_scene;

use crate::plugins::prefab::PrefabCollection;

pub struct ScenesPlugin;

impl Plugin for ScenesPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(PreStartup, startup);
    }
}

fn startup(
    mut scenes: ResMut<Assets<Scene>>,
    mut prefabs: ResMut<PrefabCollection>,
    server: Res<AssetServer>,
) {
    prefabs.insert(
        "test_scene/model.glb",
        server.load(GltfAssetLabel::Scene(0).from_asset("test_scene/model.glb")),
    );

    prefabs.insert("test_scene", scenes.add(create_test_scene()));
}
