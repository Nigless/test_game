use bevy::{
    ecs::{
        component::{self, ComponentHooks, StorageType},
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
use serde::{de::DeserializeSeed, Deserialize, Serialize};
use serde_json::{Deserializer, Value};

#[derive(Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct ResolveCollider {
    convex_hull: bool,
}

impl Component for ResolveCollider {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_add(|mut world, entity, _| {
            let (Some(convex_hull), Some(child)) = (
                world.get::<ResolveCollider>(entity).map(|c| c.convex_hull),
                world.get::<Children>(entity).map(|c| c[0]),
            ) else {
                return;
            };

            let mut shape = ComputedColliderShape::TriMesh(TriMeshFlags::all());

            if convex_hull {
                shape = ComputedColliderShape::ConvexHull
            }

            world.commands().entity(child).try_despawn_recursive();

            let (Some(mesh), Some(meshes)) = (
                world.get::<Mesh3d>(child),
                world.get_resource::<Assets<Mesh>>(),
            ) else {
                return;
            };

            let mesh = meshes.get(mesh).unwrap().clone();

            world
                .commands()
                .entity(entity)
                .try_insert(Collider::from_bevy_mesh(&mesh, &shape).unwrap())
                .remove::<Children>()
                .remove::<ResolveCollider>();
        });
    }
}

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
        hooks.on_add(|mut world, entity, _| {
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
        app.register_type::<ResolveCollider>()
            .add_systems(First, resolve.in_set(ModelSystems::Resolve));
    }
}

fn resolve(
    types_res: Res<AppTypeRegistry>,
    mut commands: Commands,
    models_q: Query<(Entity, &GltfExtras)>,
) {
    for (entity, extras) in models_q.iter() {
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
                warn!("unknown component {component_name}");
                continue;
            };

            let Some(params) = value.as_str() else {
                warn!("cant read component params {component_name}");
                continue;
            };

            let reflect_deserializer = TypedReflectDeserializer::new(&component_type, &types);

            let mut deserializer = Deserializer::from_str(params);

            let value = reflect_deserializer.deserialize(&mut deserializer).unwrap();

            commands.entity(entity).insert_reflect(value);
        }
    }
}
