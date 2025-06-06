use std::{any::TypeId, time::Duration};

use bevy::{
    color::palettes::css::ORANGE,
    ecs::{component::ComponentId, entity, world::DeferredWorld},
    pbr::NotShadowCaster,
    prelude::*,
};
use bevy_rapier3d::prelude::{
    ActiveEvents, Ccd, Collider, CollisionEvent, CollisionGroups, GravityScale, Group, LockedAxes,
    RigidBody, Sensor, Velocity,
};

use crate::{
    entities::explosion::Explosion,
    plugins::{
        billboard::BillboardMaterial,
        collision_events::CollisionStartedEvent,
        despawn::Despawn,
        health::{DamageInvokedEvent, DeadEvent, Health},
        serializable::Serializable,
    },
};

#[derive(Resource, PartialEq, Clone)]
struct FireballAssets {
    pub material: Handle<BillboardMaterial>,
    pub mesh: Handle<Mesh>,
}

#[derive(Component, Reflect, Clone)]
#[component(on_add = spawn)]
#[reflect(Component)]
#[require(Transform, Velocity)]
pub struct Fireball;

fn spawn(mut world: DeferredWorld<'_>, entity: Entity, _component_id: ComponentId) {
    let assets = world.get_resource::<FireballAssets>().cloned().unwrap();

    world
        .commands()
        .entity(entity)
        .insert_if_new((
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
            Ccd::enabled(),
            Mesh3d(assets.mesh),
            MeshMaterial3d(assets.material),
            Sensor,
        ))
        .insert(
            Serializable::default()
                .with::<Fireball>()
                .with::<Transform>()
                .with::<Despawn>()
                .with::<Velocity>(),
        );
}

pub struct FireballPlugin;

impl Plugin for FireballPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Fireball>()
            .add_systems(PreStartup, startup)
            .add_observer(explode::<CollisionStartedEvent>)
            .add_observer(explode::<DamageInvokedEvent>);
    }
}

fn startup(
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

fn explode<E: Event>(
    trigger: Trigger<E>,
    mut commands: Commands,
    entity_q: Query<(Option<&Parent>, &Transform), With<Fireball>>,
) {
    let entity = trigger.entity();

    let Ok((parent, transform)) = entity_q.get(entity) else {
        return;
    };

    commands.entity(entity).despawn_recursive();

    let explosion = commands.spawn_empty().id();

    if let Some(parent) = parent {
        commands.entity(explosion).set_parent(parent.get());
    }

    commands
        .entity(explosion)
        .insert((Explosion::new(3.0), transform.clone()));
}
