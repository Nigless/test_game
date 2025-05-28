use std::{f32::consts, ops::Deref};

use bevy::{
    ecs::{component::ComponentId, world::DeferredWorld},
    prelude::*,
};
use bevy_rapier3d::prelude::RigidBody;

use crate::{prefab::Prefab, saves::Serializable};

#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct TestScene;

pub fn create_test_scene() -> Scene {
    let mut world = World::new();

    world.spawn((Prefab::new("test_scene/model.glb"), RigidBody::Fixed));
    world.spawn((
        Name::new("directional_light"),
        Serializable::new("directional_light"),
        DirectionalLight {
            illuminance: 3000.0,
            shadows_enabled: true,
            color: Color::WHITE,
            ..default()
        },
        Transform::from_rotation(
            Quat::from_rotation_y(consts::PI * -0.1) * Quat::from_rotation_x(consts::PI * -0.6),
        ),
    ));

    world.flush();

    Scene::new(world)
}
