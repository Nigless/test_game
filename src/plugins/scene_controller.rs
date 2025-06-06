use bevy::{prelude::*, state::commands};

#[derive(Event)]
pub struct ScenePushedEvent(pub Handle<Scene>);

#[derive(Resource)]
struct SceneController {
    pub current: Entity,
}

pub struct SceneControllerPlugin;

impl Plugin for SceneControllerPlugin {
    fn build(&self, app: &mut App) {
        let entity = app
            .world_mut()
            .spawn((Name::new("scene"), Transform::default()))
            .id();

        app.insert_resource(SceneController { current: entity })
            .add_observer(handle_scene_pushed);
    }
}

fn handle_scene_pushed(
    trigger: Trigger<ScenePushedEvent>,
    scene_controller: Res<SceneController>,
    mut commands: Commands,
) {
    commands
        .entity(scene_controller.current)
        .insert(SceneRoot(trigger.event().0.clone()));
}
