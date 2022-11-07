use bevy::prelude::*;
use bevy::{
    core::Name,
    ecs::query::WorldQuery,
    hierarchy::Children,
    prelude::{App, Commands, Component, Entity, Plugin, Query, With},
};

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
        app.add_system_to_stage(CoreStage::PreUpdate, resolve);
    }
}

fn resolve(
    mut commands: Commands,
    target_q: Query<(Entity, &Children), With<WithHead>>,
    entity_q: Query<(Option<&Children>, Option<&Name>), Without<WithHead>>,
) {
    let label = "#head";

    for (entity, children) in target_q.iter() {
        let mut head = None;
        for child in children.into_iter() {
            if let Some(entity) = find(&entity_q, *child, &label) {
                head = Some(entity)
            }
        }
        commands
            .entity(entity)
            .remove::<WithHead>()
            .insert(Head::new(head.unwrap()));
    }
}

fn find<Q: WorldQuery>(
    target_q: &Query<(Option<&Children>, Option<&Name>), Q>,
    entity: Entity,
    label: &str,
) -> Option<Entity> {
    let (children, name) = match target_q.get(entity) {
        Ok(e) => e,
        Err(_) => return None,
    };

    if let Some(_name) = name {
        if _name.as_str().contains(label) {
            return Some(entity);
        }
    };

    for child in children?.into_iter() {
        if let Some(result) = find(target_q, *child, label) {
            return Some(result);
        }
    }

    None
}
