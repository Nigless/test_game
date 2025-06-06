use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::format,
    fs::{self, File},
    io::{Read, Write},
    path::Path,
    sync::RwLockReadGuard,
    time::Instant,
};

use bevy::{
    app::{App, Plugin},
    ecs::{component::Component, system::Query},
    log::tracing_subscriber::fmt::MakeWriter,
    prelude::*,
    reflect::{serde::TypedReflectSerializer, TypeRegistry},
};
use serde_json::{json, Map};
use serde_json::{Deserializer, Value};
use uuid::Uuid;

use crate::plugins::prefab::{
    self, insert_reflect, parse_component, spawn_prefab_recursive, Prefab, PrefabCollection,
};
use chrono::{DateTime, NaiveDateTime, Utc};

use bevy::{
    ecs::{
        component::{self, ComponentHooks, StorageType},
        reflect::ReflectCommandExt,
    },
    prelude::*,
    ptr::OwningPtr,
    reflect::{serde::ReflectDeserializer, TypeData, TypeRegistration},
};
use bevy_rapier3d::{
    geometry::{Collider, ComputedColliderShape},
    prelude::{TriMeshFlags, VHACDParameters},
};
use serde::{de::DeserializeSeed, Deserialize, Serialize};

const FILE_PATTERN: &str = "%Y-%m-%dT%H:%M:%S%.3f.json";

#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct Serializable {
    id: String,
    parent: Option<Entity>,
    component_types: Vec<TypeId>,
}

impl Default for Serializable {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            parent: default(),
            component_types: default(),
        }
    }
}

impl Serializable {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_owned(),
            ..default()
        }
    }

    pub fn with<T: ?Sized + 'static + Reflect>(mut self) -> Self {
        self.component_types.push(TypeId::of::<T>());
        self
    }

    pub fn set_parent(&mut self, entity: Entity) {
        self.parent = Some(entity);
    }
}

pub struct SerializablePlugin;

impl Plugin for SerializablePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(First, (save, load).chain())
            .add_systems(First, startup)
            .register_type::<Serializable>();
    }
}

fn startup() {
    let path = Path::new("saves");

    if path.exists() {
        return;
    }

    fs::create_dir_all(path).unwrap();
}

fn save(
    world: &World,
    entity_q: Query<(Entity, &Serializable)>,
    types_res: Res<AppTypeRegistry>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if !keyboard.just_pressed(KeyCode::F5) {
        return;
    }

    let types = types_res.read();

    let mut save_file = SaveFile::new();

    for (entity, serialize) in entity_q.iter() {
        let mut components = HashMap::<String, Value>::new();

        for type_id in serialize.component_types.iter().cloned() {
            let Ok(reflect) = world.get_reflect(entity, type_id) else {
                continue;
            };

            let serializer = TypedReflectSerializer::new(reflect, &types);

            let output = serde_json::to_value(&serializer).unwrap();

            let type_path = types.get_type_info(type_id).unwrap().type_path().to_owned();

            components.insert(type_path, output);
        }

        let mut id: Option<String> = None;
        let mut current = entity;

        while let Some(serializable) = world.entity(current).get::<Serializable>() {
            id = id
                .map(|id| format!("{}:{}", serializable.id, id))
                .or(Some(serializable.id.clone()));

            let Some(parent) = serializable.parent else {
                break;
            };

            current = parent;
        }

        let Some(id) = id else {
            continue;
        };

        if serialize.parent.is_none() {
            save_file.entities.push(id.clone());
        }

        save_file.entities_data.insert(id.clone(), components);
    }

    if save_file.entities_data.is_empty() {
        warn!("no entities to save");
        return;
    }

    let now = chrono::Local::now().format(FILE_PATTERN);

    let file_path = Path::new("saves").join(now.to_string());

    let file = File::create(&file_path).unwrap();

    serde_json::to_writer(file.make_writer(), &save_file).unwrap();
}

fn get_id(world: &mut World, entity: Entity) -> Option<String> {
    let mut id: Option<String> = None;
    let mut current = entity;

    while let Some(serializable) = world.entity(current).get::<Serializable>() {
        id = id
            .map(|id| format!("{}:{}", serializable.id, id))
            .or(Some(serializable.id.clone()));

        let Some(parent) = serializable.parent else {
            break;
        };

        current = parent;
    }

    id
}

fn get_components(
    types: &TypeRegistry,
    save_file: &SaveFile,
    id: String,
) -> (Vec<Box<dyn PartialReflect>>, Vec<TypeId>) {
    let Some(json_object) = save_file.entities_data.get(&id) else {
        return (vec![], vec![]);
    };

    let mut components = vec![];

    let mut component_types = Vec::<TypeId>::new();

    for (component_name, value) in json_object.clone() {
        let Some(component_type) = types.get_with_type_path(&component_name).cloned() else {
            warn!("unknown component {component_name}");
            continue;
        };
        component_types.push(component_type.type_id());

        let component = parse_component(&types, &component_type, value.to_string());

        components.push(component);
    }

    (components, component_types)
}

fn spawn_scene(world: &mut World, save_file: SaveFile) {
    let types = world.resource::<AppTypeRegistry>().clone();
    let types = types.read();

    for id in save_file.entities.clone() {
        let entity = world.spawn_empty().id();

        let (components, component_types) = get_components(&types, &save_file, id.clone());

        for component in components {
            insert_reflect(world, entity, component);
        }

        world.entity_mut(entity).insert(Serializable {
            id: id.clone(),
            component_types,
            ..default()
        });

        spawn_prefab_recursive(world, entity, &|world, entity| {
            let Some(id) = get_id(world, entity) else {
                return;
            };

            for component in get_components(&types, &save_file, id).0 {
                insert_reflect(world, entity, component);
            }
        });
    }
}

fn load_file(world: &mut World, save_file: SaveFile) -> Scene {
    let mut scene_world = World::new();

    scene_world.insert_resource(world.remove_resource::<SceneSpawner>().unwrap());
    scene_world.insert_resource(world.remove_resource::<PrefabCollection>().unwrap());
    scene_world.insert_resource(world.remove_resource::<Assets<Scene>>().unwrap());

    spawn_scene(&mut scene_world, save_file);

    world.insert_resource(scene_world.remove_resource::<SceneSpawner>().unwrap());
    world.insert_resource(scene_world.remove_resource::<PrefabCollection>().unwrap());
    world.insert_resource(scene_world.remove_resource::<Assets<Scene>>().unwrap());

    Scene::new(scene_world)
}

#[derive(Serialize, Deserialize, Default)]
pub struct SaveFile {
    date: DateTime<Utc>,
    entities: Vec<String>,
    entities_data: HashMap<String, HashMap<String, Value>>,
}

impl SaveFile {
    fn new() -> Self {
        Self {
            date: Utc::now(),
            ..default()
        }
    }
}

fn load(keyboard: Res<ButtonInput<KeyCode>>, mut commands: Commands) {
    if !keyboard.just_pressed(KeyCode::F6) {
        return;
    }

    let mut files = Vec::<SaveFile>::new();

    for entry in fs::read_dir("saves").unwrap() {
        let path = entry.unwrap().path();

        if !path.extension().map(|e| e == "json").unwrap_or(false) {
            continue;
        };

        let Some(file_path) = path.to_str() else {
            continue;
        };

        let content = fs::read_to_string(file_path).unwrap();

        let Ok(save_file) = serde_json::from_str::<SaveFile>(&content) else {
            return;
        };

        files.push(save_file);
    }

    let mut last_save = files.remove(0);

    for save_file in files.drain(0..) {
        if last_save.date < save_file.date {
            last_save = save_file
        }
    }

    commands.queue(|world: &mut World| {
        load_file(world, last_save);
    });
}
