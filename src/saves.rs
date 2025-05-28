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

use crate::{
    control::Input,
    prefab::{
        self, insert_reflect, parse_component, spawn_prefab_recursive, Prefab, PrefabCollection,
    },
    scenes::Scenes,
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

    pub fn with<T: ?Sized + 'static>(mut self) -> Self {
        self.component_types.push(TypeId::of::<T>());
        self
    }

    pub fn set_parent(&mut self, entity: Entity) {
        self.parent = Some(entity);
    }
}

pub struct SavesPlugin;

impl Plugin for SavesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, (save, load).chain())
            .add_systems(PreStartup, startup)
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
            save_file.entities_id.push(id.clone());
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

fn get_components(
    types: &TypeRegistry,
    save_file: &SaveFile,
    id: String,
) -> Vec<Box<dyn PartialReflect>> {
    let Some(json_object) = save_file.entities_data.get(&id) else {
        return vec![];
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

    components.push(Box::new(Serializable {
        id,
        component_types,
        ..default()
    }));

    components
}

fn load_file(save_file: SaveFile) -> impl Command {
    move |world: &mut World| {
        let root_scene = world.resource::<Scenes>().current;

        world.entity_mut(root_scene).despawn_descendants();

        let types = world.resource::<AppTypeRegistry>().clone();
        let types = types.read();

        for root_id in save_file.entities_id.clone() {
            let entity = world.spawn_empty().set_parent(root_scene).id();

            for component in get_components(&types, &save_file, root_id) {
                insert_reflect(world, entity, component);
            }

            spawn_prefab_recursive(world, entity, &|world, entity| {
                let serializable_id = world.get::<Serializable>(entity).map(|s| s.id.clone());

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
                    return;
                };

                let parent = world.get::<Serializable>(entity).and_then(|s| s.parent);

                for component in get_components(&types, &save_file, id) {
                    insert_reflect(world, entity, component);
                }

                if let Some(mut serializable) = world.get_mut::<Serializable>(entity) {
                    serializable.id = serializable_id.unwrap_or(serializable.id.clone());
                    serializable.parent = parent;
                };
            });
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct SaveFile {
    date: DateTime<Utc>,
    entities_id: Vec<String>,
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

    commands.queue(load_file(last_save));
}
