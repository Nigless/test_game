use std::f32::consts;

use bevy::prelude::*;
use bevy_rapier3d::prelude::RigidBody;

use crate::{model::Model, with_child::WithChild};

#[derive(Bundle)]
pub struct TestLevelBundle {
    name: Name,
    model: Model,
    body: RigidBody,
    light: WithChild<LightBundle>,
}

impl Default for TestLevelBundle {
    fn default() -> Self {
        Self {
            name: Name::new("test_scene"),
            model: Model::new("test_scene/model.glb"),
            body: RigidBody::Fixed,
            light: default(),
        }
    }
}

#[derive(Bundle)]
struct LightBundle {
    name: Name,
    directional_light: DirectionalLight,
    transform: Transform,
}

impl Default for LightBundle {
    fn default() -> Self {
        Self {
            name: Name::new("directional_light"),
            directional_light: DirectionalLight {
                illuminance: 3000.0,
                shadows_enabled: true,
                color: Color::WHITE,
                ..default()
            },
            transform: Transform::from_rotation(
                Quat::from_rotation_y(consts::PI * -0.1) * Quat::from_rotation_x(consts::PI * -0.6),
            ),
        }
    }
}
