use bevy::prelude::*;
use bevy_rapier3d::{prelude::*, rapier::prelude::CollisionEventFlags};

#[derive(Event)]
pub struct CollisionStartedEvent(Entity, CollisionEventFlags);

#[derive(Event)]
pub struct CollisionStoppedEvent(Entity, CollisionEventFlags);

pub struct CollisionEventsPlugins;

impl Plugin for CollisionEventsPlugins {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(First, resolve_events);
    }
}

fn resolve_events(mut commands: Commands, mut collisions: EventReader<CollisionEvent>) {
    for event in collisions.read() {
        match event {
            CollisionEvent::Started(entity_a, entity_b, flags) => {
                if let Some(mut entity) = commands.get_entity(*entity_a) {
                    entity.trigger(CollisionStartedEvent(*entity_b, *flags));
                }

                if let Some(mut entity) = commands.get_entity(*entity_b) {
                    entity.trigger(CollisionStartedEvent(*entity_a, *flags));
                }
            }

            CollisionEvent::Stopped(entity_a, entity_b, flags) => {
                if let Some(mut entity) = commands.get_entity(*entity_a) {
                    entity.trigger(CollisionStoppedEvent(*entity_b, *flags));
                }

                if let Some(mut entity) = commands.get_entity(*entity_b) {
                    entity.trigger(CollisionStoppedEvent(*entity_a, *flags));
                }
            }
        }
    }
}
