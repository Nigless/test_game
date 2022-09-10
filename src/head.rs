use core::panic;
use std::process::Child;

use bevy::{
    core::Name,
    ecs::world::{self, WorldBorrow},
    hierarchy::Children,
    prelude::{App, Commands, Component, Entity, Plugin, Query, With, World},
};
use regex::Regex;

#[derive(Component)]
pub struct WithHead;

#[derive(Component)]
pub struct Head {
    pub target: Entity,
}

impl Head {
    pub fn new(target: Entity) -> Self {
        Self { target }
    }
}

pub struct HeadPlugin;

impl Plugin for HeadPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(resolve);
    }
}

fn resolve(world: &World, mut commands: Commands, entity: Query<Entity, With<WithHead>>) {
    for entity in entity.iter() {
        let head = get_head(world, entity, &Regex::new(r"#head").unwrap());
        if let None = head {
            return;
        }
        commands
            .entity(entity)
            .remove::<WithHead>()
            .insert(Head::new(head.unwrap()));
    }
}

fn get_head(world: &World, entity: Entity, regex: &Regex) -> Option<Entity> {
    for entity in world.get::<Children>(entity)?.into_iter() {
        if let Some(name) = world.get::<Name>(*entity) {
            if regex.is_match(name.as_str()) {
                return Some(*entity);
            }
        }
        let head = get_head(world, *entity, regex);
        if let Some(head) = head {
            return Some(head);
        }
    }
    None
}
