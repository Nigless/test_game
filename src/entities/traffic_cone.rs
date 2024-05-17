use bevy::{
    asset::Handle,
    core::Name,
    ecs::bundle::Bundle,
    pbr::StandardMaterial,
    render::view::{InheritedVisibility, ViewVisibility, Visibility},
    transform::TransformBundle,
};
use bevy_rapier3d::dynamics::{RigidBody, Velocity};

use crate::model::Model;

#[derive(Bundle)]
pub struct TrafficCone {
    name: Name,
    velocity: Velocity,
    body: RigidBody,
    model: Model,
    transform: TransformBundle,
    material: Handle<StandardMaterial>,

    visibility: Visibility,
    inherited_visibility: InheritedVisibility,
    view_visibility: ViewVisibility,
}

impl TrafficCone {
    pub fn new() -> Self {
        Self {
            name: Name::new("Traffic cone"),
            body: RigidBody::Dynamic,
            model: Model::new("traffic_cone/model.glb"),
            velocity: Velocity::default(),
            transform: TransformBundle::default(),
            material: Handle::default(),

            visibility: Visibility::default(),
            inherited_visibility: InheritedVisibility::default(),
            view_visibility: ViewVisibility::default(),
        }
    }
}
