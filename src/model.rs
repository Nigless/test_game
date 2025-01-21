use bevy::{
    ecs::{
        component::{ComponentHooks, StorageType},
        reflect::ReflectCommandExt,
    },
    prelude::*,
    ptr::OwningPtr,
    reflect::{
        serde::{ReflectDeserializer, TypedReflectDeserializer},
        TypeData, TypeRegistration,
    },
};
use bevy_rapier3d::{
    geometry::{Collider, ComputedColliderShape},
    prelude::{TriMeshFlags, VHACDParameters},
};
use serde::de::DeserializeSeed;
use serde_json::{Deserializer, Value};

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

impl Component for Model {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_add(|mut world, entity, _component_id| {
            let src = world.get_mut::<Model>(entity).unwrap().src.clone();

            world.commands().entity(entity).remove::<Model>();

            let server = world.get_resource::<AssetServer>().unwrap();

            let scene_root = SceneRoot(server.load(GltfAssetLabel::Scene(0).from_asset(src)));

            world
                .commands()
                .entity(entity)
                .remove::<Model>()
                .insert(scene_root);
        });
    }
}

#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub enum ModelSystems {
    Resolve,
}

pub struct ModelPlugin;

impl Plugin for ModelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(First, resolve.in_set(ModelSystems::Resolve));
    }
}

fn resolve(
    meshes_res: Res<Assets<Mesh>>,
    types_res: Res<AppTypeRegistry>,
    mut commands: Commands,
    models_q: Query<(Entity, &Children, &GltfExtras)>,
    meshes_q: Query<&Mesh3d>,
) {
    for (entity, children, extras) in models_q.iter() {
        commands.entity(entity).remove::<GltfExtras>();

        let Ok(json_value) = serde_json::from_str::<Value>(&extras.value) else {
            continue;
        };

        let Some(extras) = json_value.as_object() else {
            continue;
        };

        let types = types_res.read();

        for (component_name, value) in extras.clone() {
            let Some(component_type) = types.get_with_type_path(&component_name).cloned() else {
                continue;
            };

            let Some(params) = value.as_str() else {
                continue;
            };

            let reflect_deserializer = TypedReflectDeserializer::new(&component_type, &types);

            let mut deserializer = Deserializer::from_str(params);

            let value = reflect_deserializer.deserialize(&mut deserializer).unwrap();

            commands.entity(entity).insert_reflect(value);
        }

        if !extras
            .get("collider")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            continue;
        }

        let mesh = meshes_q.get(children[0]).unwrap();

        let mesh = meshes_res.get(mesh).unwrap();

        commands.entity(children[0]).despawn();

        commands
            .entity(entity)
            .insert(
                Collider::from_bevy_mesh(
                    mesh,
                    &ComputedColliderShape::TriMesh(TriMeshFlags::all()),
                )
                .unwrap(),
            )
            .remove::<Children>();
    }
}
