use bevy::prelude::*;
use bevy::{
    app::{App, Plugin},
    prelude::{Component, Entity},
    reflect::Reflect,
    utils::hashbrown::HashMap,
};

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Linker {
    links: HashMap<String, Entity>,
}

impl Linker {
    pub fn new() -> Self {
        Self {
            links: HashMap::new(),
        }
    }

    pub fn with_link(mut self, name: &str, entity: Entity) -> Self {
        self.links.insert(name.to_owned(), entity);
        self
    }

    pub fn get(&self, name: &str) -> Option<&Entity> {
        self.links.get(name)
    }
}

pub struct LinkerPlugin;

impl Plugin for LinkerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Linker>();
    }
}
