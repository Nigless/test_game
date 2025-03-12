use std::time::Duration;

use bevy::{color::palettes::css::ORANGE, ecs::entity, pbr::NotShadowCaster, prelude::*};
use bevy_rapier3d::prelude::{ActiveEvents, Collider, CollisionEvent, Sensor};

use crate::{billboard::BillboardMaterial, despawn::Despawn, library::Spawnable};

#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
#[require(Transform)]
pub struct Fireball;

impl Spawnable for Fireball {
    fn spawn(&self, commands: &mut Commands) -> Entity {
        commands
            .spawn(self.clone())
            .queue(|entity: Entity, world: &mut World| {
                let texture_handle = {
                    let asset_server = world.get_resource::<AssetServer>().unwrap();
                    asset_server.load("fireball/texture.png")
                };

                let quad_handle = {
                    let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
                    meshes.add(Rectangle::new(1.0, 1.0))
                };

                let material_handle: Handle<BillboardMaterial> = {
                    let mut materials = world
                        .get_resource_mut::<Assets<BillboardMaterial>>()
                        .unwrap();

                    materials.add(BillboardMaterial::new(texture_handle.clone()))
                };

                let mut commands = world.commands();

                commands.entity(entity).insert((
                    Name::new("fireball"),
                    Mesh3d(quad_handle.clone()),
                    MeshMaterial3d(material_handle),
                    NotShadowCaster,
                    Sensor,
                    Collider::ball(0.3),
                    ActiveEvents::COLLISION_EVENTS,
                    PointLight {
                        intensity: 100_000.0,
                        color: ORANGE.into(),
                        shadows_enabled: true,
                        ..default()
                    },
                ));
            })
            .id()
    }
}

pub struct FireballPlugin;

impl Plugin for FireballPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Despawn>()
            .add_systems(FixedPreUpdate, update);
    }
}

fn update(
    mut commands: Commands,
    mut collisions: EventReader<CollisionEvent>,
    fireball_q: Query<(Option<&Parent>, &Transform), With<Fireball>>,
) {
    for event in collisions.read() {
        let CollisionEvent::Started(entity_a, entity_b, _) = event else {
            continue;
        };

        let fireball = if fireball_q.contains(*entity_a) {
            entity_a
        } else if fireball_q.contains(*entity_b) {
            entity_b
        } else {
            continue;
        };

        commands.entity(*fireball).despawn_recursive();
    }
}
