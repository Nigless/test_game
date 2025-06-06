use bevy::ecs::{event::Event, system::EntityCommand};
use bevy::prelude::*;

pub fn trigger<E: Event>(event: E) -> impl EntityCommand {
    |entity: Entity, world: &mut World| {
        world.commands().entity(entity).trigger(event);
    }
}
