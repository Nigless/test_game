use std::{any::Any, collections::HashMap, io::Read};

use bevy::{
    app::Plugin,
    ecs::{
        component::{Component, ComponentId},
        observer::TriggerTargets,
        reflect::ReflectBundle,
        system::Query,
        world::DeferredWorld,
    },
    gltf::{GltfMaterialExtras, GltfMeshExtras, GltfSceneExtras},
    prelude::*,
    reflect::{serde::TypedReflectDeserializer, TypeRegistry},
    scene::InstanceId,
};
use bevy_rapier3d::prelude::{Collider, ComputedColliderShape, TriMeshFlags};
use serde_json::{json, Map};
use serde_json::{Deserializer, Value};
use uuid::Uuid;

use crate::{control::Input, saves::Serializable, scenes::Scenes};
use chrono::{DateTime, NaiveDateTime, Utc};

use bevy::reflect::TypeRegistration;
use serde::de::DeserializeSeed;

pub fn insert_reflect(world: &mut World, entity: Entity, component: Box<dyn PartialReflect>) {
    let type_registry = world.resource::<AppTypeRegistry>().clone();
    let type_registry = type_registry.read();

    let type_info = component
        .get_represented_type_info()
        .expect("component should represent a type.");
    let type_path = type_info.type_path();
    let Ok(mut entity) = world.get_entity_mut(entity) else {
        panic!("error[B0003]: Could not insert a reflected component (of type {type_path}) for entity {entity:?} because it doesn't exist in this World. See: https://bevyengine.org/learn/errors/b0003");
    };
    let Some(type_registration) = type_registry.get(type_info.type_id()) else {
        panic!("`{type_path}` should be registered in type registry via `App::register_type<{type_path}>`");
    };

    if let Some(reflect_component) = type_registration.data::<ReflectComponent>() {
        reflect_component.insert(&mut entity, component.as_partial_reflect(), &type_registry);
    } else if let Some(reflect_bundle) = type_registration.data::<ReflectBundle>() {
        reflect_bundle.insert(&mut entity, component.as_partial_reflect(), &type_registry);
    } else {
        panic!("`{type_path}` should have #[reflect(Component)] or #[reflect(Bundle)]");
    }
}

pub fn parse_component(
    types: &TypeRegistry,
    type_reg: &TypeRegistration,
    value: String,
) -> Box<dyn PartialReflect> {
    let reflect_deserializer = TypedReflectDeserializer::new(type_reg, &types);

    let mut deserializer = Deserializer::from_str(&value);

    reflect_deserializer.deserialize(&mut deserializer).unwrap()
}

fn resolve_gltf(world: &mut World, entity: Entity) {
    world.entity_mut(entity).remove::<GltfSceneExtras>();
    world.entity_mut(entity).remove::<GltfMeshExtras>();
    world.entity_mut(entity).remove::<GltfMaterialExtras>();

    let Some(extras) = world.entity_mut(entity).take::<GltfExtras>() else {
        return;
    };

    let Ok(json_value) = serde_json::from_str::<Value>(&extras.value) else {
        return;
    };

    let Some(extras) = json_value.as_object() else {
        return;
    };

    let type_registry = world.resource::<AppTypeRegistry>().clone();
    let type_registry = type_registry.read();

    for (component_name, value) in extras.clone() {
        let Some(component_type) = type_registry.get_with_type_path(&component_name).cloned()
        else {
            warn!("unknown component {component_name}");
            continue;
        };

        let component = parse_component(
            &type_registry,
            &component_type,
            value.as_str().unwrap().to_string(),
        );

        insert_reflect(world, entity, component);
    }
}

fn spawn_prefab(world: &mut World, entity: Entity) -> Vec<Entity> {
    let Some(mut prefab) = world.entity(entity).get::<Prefab>().cloned() else {
        return vec![];
    };

    let scene = world
        .resource::<PrefabCollection>()
        .get(&prefab.name)
        .expect(&format!("unable to get prefab: \"{}\"", prefab.name));

    let instance_id = world
        .resource_scope(|world, mut spawner: Mut<SceneSpawner>| spawner.spawn_sync(world, &scene))
        .expect("unable to spawn scene");

    prefab.instance_id = Some(instance_id);

    world.entity_mut(entity).insert(prefab);

    let entities = world
        .resource::<SceneSpawner>()
        .iter_instance_entities(instance_id)
        .collect::<Vec<_>>();

    for child in &entities {
        if !world.entity(child.clone()).contains::<Parent>() {
            world.entity_mut(child.clone()).set_parent(entity);
        }
    }

    entities
}

pub fn spawn_prefab_recursive(
    world: &mut World,
    entity: Entity,
    resolve_entity: &impl Fn(&mut World, Entity),
) {
    for child in spawn_prefab(world, entity) {
        resolve_gltf(world, child);

        world
            .entity_mut(child)
            .get_mut::<Serializable>()
            .map(|mut s| s.set_parent(entity));

        resolve_entity(world, child);
        spawn_prefab_recursive(world, child, resolve_entity);
    }
}

#[derive(Resource, Default, Reflect, PartialEq)]
#[reflect(Resource)]
pub struct PrefabCollection {
    items: HashMap<String, Handle<Scene>>,
}

impl PrefabCollection {
    pub fn insert(&mut self, key: &str, scene: Handle<Scene>) {
        self.items.insert(key.to_owned(), scene);
    }

    pub fn get(&self, key: &str) -> Option<Handle<Scene>> {
        self.items.get(key).cloned()
    }
}

#[derive(Reflect, Component)]
#[component(on_add = resolve_collider)]
#[reflect(Component)]
pub struct ResolveCollider {
    convex_hull: bool,
}

fn resolve_collider(mut world: DeferredWorld<'_>, entity: Entity, _: ComponentId) {
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
        .remove::<ResolveCollider>()
        .remove::<Children>();
}

#[derive(Component, Default, Reflect, PartialEq, Clone, Debug)]
#[require(Transform)]
#[component(on_add = resolve_prefab)]
#[reflect(Component, Default)]
pub struct Prefab {
    pub name: String,
    #[reflect(skip_serializing)]
    pub instance_id: Option<InstanceId>,
}

impl Prefab {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            ..default()
        }
    }
}

fn resolve_prefab(mut world: DeferredWorld<'_>, entity: Entity, _: ComponentId) {
    world.commands().queue(move |world: &mut World| {
        let prefab = world.entity(entity).get::<Prefab>().cloned().unwrap();

        world
            .entity_mut(entity)
            .insert_if_new(Name::new(format!("prefab:{}", prefab.name)));

        if !world.contains_resource::<PrefabCollection>() {
            return;
        }

        if !world.contains_resource::<SceneSpawner>() {
            return;
        }

        if prefab.instance_id.is_some() {
            return;
        }

        spawn_prefab_recursive(world, entity, &|_, _| {});
    });
}

pub struct PrefabPlugin;

impl Plugin for PrefabPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.register_type::<PrefabCollection>()
            .insert_resource(PrefabCollection::default())
            .register_type::<Prefab>()
            .register_type::<ResolveCollider>();
    }
}
