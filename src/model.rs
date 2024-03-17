use bevy::{gltf::GltfMesh, prelude::*};

#[derive(Component)]
pub struct Model {
    pub src: String,
}

impl Model {
    pub fn new<'a>(src: &'a str) -> Self {
        Self {
            src: src.to_owned(),
        }
    }
}

pub struct ModelPlugin;

impl Plugin for ModelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(First, resolve);
    }
}

fn resolve(server: Res<AssetServer>, mut commands: Commands, models_q: Query<(Entity, &Model)>) {
    for (entity, model) in models_q.iter() {
        let src = model.src.clone();

        commands
            .entity(entity)
            .remove::<Model>()
            .insert(server.load::<Mesh>(src));
    }
}
