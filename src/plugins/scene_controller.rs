use std::{mem::take, ops::Deref};

use bevy::{prelude::*, state::commands};
use bevy_rapier3d::parry::simba::scalar::SupersetOf;

#[derive(Resource)]
pub struct SceneController {
    pub current: Entity,
    new_scene: Option<Scene>,
}

impl SceneController {
    pub fn push(&mut self, scene: Scene) {
        self.new_scene = Some(scene)
    }

    fn new(entity: Entity) -> Self {
        Self {
            current: entity,
            new_scene: default(),
        }
    }
}

pub struct SceneControllerPlugin;

impl Plugin for SceneControllerPlugin {
    fn build(&self, app: &mut App) {
        let entity = app
            .world_mut()
            .spawn((Name::new("scene"), Transform::default()))
            .id();

        app.insert_resource(SceneController::new(entity))
            .add_systems(First, resolve);
    }
}

fn resolve(
    mut scene_controller: ResMut<SceneController>,
    entity_q: Query<Option<&SceneRoot>>,
    mut scenes: ResMut<Assets<Scene>>,
    mut commands: Commands,
) {
    let Some(scene) = scene_controller.new_scene.take() else {
        return;
    };

    if let Some(scene_root) = entity_q.get(scene_controller.current).ok().and_then(|s| s) {
        scenes.remove(scene_root.0.id());
    };

    let handle_scene = scenes.add(scene);

    commands
        .entity(scene_controller.current)
        .insert(SceneRoot(handle_scene));
}
