use std::f32::consts;

use bevy::prelude::*;
use bevy_rapier3d::prelude::{RigidBody, Velocity};

use crate::{
    camera_controller::Spectate,
    control::Control,
    entities::{block::BlockBundle, fireball::Fireball, player::Player},
    library::Spawnable,
    model::Model,
    with_entity::WithEntity,
};

pub struct TestLevelBundle;

impl TestLevelBundle {
    fn bundle() -> impl Bundle {
        (
            Name::new("test_scene"),
            Model::new("test_scene/model.glb"),
            RigidBody::Fixed,
        )
    }
}

impl Spawnable for TestLevelBundle {
    fn spawn<'a>(&self, commands: &'a mut Commands) -> EntityCommands<'a> {
        let entity = commands.spawn(Self::bundle()).id();

        commands
            .spawn((
                Name::new("directional_light"),
                DirectionalLight {
                    illuminance: 3000.0,
                    shadows_enabled: true,
                    color: Color::WHITE,
                    ..default()
                },
                Transform::from_rotation(
                    Quat::from_rotation_y(consts::PI * -0.1)
                        * Quat::from_rotation_x(consts::PI * -0.6),
                ),
            ))
            .set_parent(entity);

        Player
            .spawn(commands)
            .insert((Transform::from_xyz(0.0, 3.0, 0.0), Spectate, Control))
            .set_parent(entity);

        Fireball
            .spawn(commands)
            .insert((Transform::from_xyz(0.0, 1.0, 3.0),))
            .insert(Velocity::linear(Vec3::X))
            .set_parent(entity);

        Fireball
            .spawn(commands)
            .insert((Transform::from_xyz(0.0, 1.0, -3.0),))
            .set_parent(entity);

        BlockBundle::default()
            .with_transform(Transform::from_xyz(-4.0, 3.0, 24.0))
            .spawn(commands)
            .set_parent(entity);

        BlockBundle::default()
            .with_transform(Transform::from_xyz(4.0, 3.0, 16.0))
            .spawn(commands)
            .set_parent(entity);

        BlockBundle::new(1.0, 0.5, 4.0)
            .with_transform(Transform::from_xyz(4.0, 3.0, 24.0))
            .with_mass(100.0)
            .spawn(commands)
            .set_parent(entity);

        BlockBundle::new(0.5, 0.5, 0.5)
            .with_mass(25.0 / 2.0)
            .with_transform(Transform::from_xyz(0.0, 2.0, 20.0))
            .spawn(commands)
            .set_parent(entity);

        commands.entity(entity)
    }
}
