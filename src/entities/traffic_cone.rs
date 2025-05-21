use bevy::{
    core::Name,
    ecs::system::{Commands, EntityCommands},
    transform::components::Transform,
};
use bevy_rapier3d::prelude::{ColliderMassProperties, RigidBody};

use crate::{library::Spawnable, liquid::VolumeScale, model::Model, with_mesh::WithMesh};

pub struct TrafficCone;

impl Spawnable for TrafficCone {
    fn spawn<'a>(&self, commands: &'a mut Commands) -> EntityCommands<'a> {
        commands.spawn((
            Name::new("traffic_cone"),
            Model::new("traffic_cone/model.glb"),
            VolumeScale::new(0.001),
            Transform::default(),
            RigidBody::Dynamic,
            ColliderMassProperties::Mass(3.0),
        ))
    }
}
