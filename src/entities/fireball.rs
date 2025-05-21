use std::time::Duration;

use bevy::{color::palettes::css::ORANGE, ecs::entity, pbr::NotShadowCaster, prelude::*};
use bevy_rapier3d::prelude::{
    ActiveEvents, Collider, CollisionEvent, GravityScale, LockedAxes, RigidBody, Sensor,
};

use crate::{
    billboard::BillboardMaterial, despawn::Despawn, explosion::Explosion, library::Spawnable,
    AppSystems,
};

#[derive(Resource, PartialEq, Clone)]
struct FireballAssets {
    pub material: Handle<BillboardMaterial>,
    pub mesh: Handle<Mesh>,
}

#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
#[require(Transform)]
pub struct Fireball;

impl Spawnable for Fireball {
    fn spawn<'a>(&self, commands: &'a mut Commands) -> EntityCommands<'a> {
        let entity = commands
            .spawn((
                self.clone(),
                Name::new("fireball"),
                LockedAxes::ROTATION_LOCKED,
                NotShadowCaster,
                RigidBody::Dynamic,
                GravityScale(0.0),
                Collider::ball(0.3),
                ActiveEvents::COLLISION_EVENTS,
                PointLight {
                    intensity: 100_000.0,
                    color: ORANGE.into(),
                    shadows_enabled: true,
                    ..default()
                },
            ))
            .queue(|entity: Entity, world: &mut World| {
                let assets = world.get_resource::<FireballAssets>().cloned().unwrap();

                world
                    .commands()
                    .entity(entity)
                    .insert((Mesh3d(assets.mesh), MeshMaterial3d(assets.material)));
            })
            .id();

        commands.entity(entity)
    }
}

pub struct FireballPlugin;

impl Plugin for FireballPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Despawn>()
            .add_systems(FixedPreUpdate, update)
            .add_systems(PreStartup, load);
    }
}

fn load(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BillboardMaterial>>,
) {
    let texture = asset_server.load("fireball/texture.png");
    let mesh = meshes.add(Rectangle::new(1.0, 1.0));
    let material = materials.add(BillboardMaterial::new(texture.clone()));

    commands.insert_resource(FireballAssets { mesh, material });
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

        let Ok((_, transform)) = fireball_q.get(*fireball) else {
            continue;
        };

        Explosion::new(10.0)
            .spawn(&mut commands)
            .insert(transform.clone());

        commands.entity(*fireball).despawn_recursive();
    }
}
