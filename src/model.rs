use bevy::prelude::*;

#[derive(Component)]
pub struct WithModel {
    pub src: String,
}

impl WithModel {
    pub fn new<'a>(src: &'a str) -> Self {
        Self {
            src: src.to_owned(),
        }
    }
}

#[derive(Component)]
pub struct Model;

pub struct ModelPlugin;

impl Plugin for ModelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, resolve);
    }
}

fn resolve(
    server: Res<AssetServer>,
    mut commands: Commands,
    models_q: Query<(Entity, &WithModel)>,
) {
    for (entity, model) in models_q.iter() {
        let src = model.src.clone();

        commands
            .entity(entity)
            .remove::<WithModel>()
            .insert(Model)
            .insert(server.load::<Scene>(src));
    }
}
