use std::f32::consts;

use bevy::prelude::*;
use bevy_rapier3d::prelude::RigidBody;

use crate::{library::Spawnable, model::Model, with_child::WithChild};

pub struct TestLevelBundle;

impl TestLevelBundle {
    pub fn bundle() -> impl Bundle {
        (
            Name::new("test_scene"),
            Model::new("test_scene/model.glb"),
            RigidBody::Fixed,
            WithChild::new(LightBundle),
        )
    }
}

struct LightBundle;

impl Spawnable for LightBundle {
    fn spawn(&self, commands: &mut Commands) -> Entity {
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
            .id()
    }
}
