use bevy::{
    asset::Handle,
    core::Name,
    ecs::bundle::Bundle,
    pbr::StandardMaterial,
    prelude::default,
    render::view::{InheritedVisibility, ViewVisibility, Visibility},
    transform::TransformBundle,
};
use bevy_rapier3d::dynamics::{RigidBody, Velocity};

use crate::{hit_box::HitBox, model::Model};

#[derive(Bundle)]
pub struct TrafficCone {
    name: Name,
    velocity: Velocity,
    body: RigidBody,
    model: Model,
    hit_box: HitBox,
    transform: TransformBundle,
    material: Handle<StandardMaterial>,

    visibility: Visibility,
    inherited_visibility: InheritedVisibility,
    view_visibility: ViewVisibility,
}

impl TrafficCone {
    pub fn new() -> Self {
        Self {
            name: Name::new("TrafficCone"),
            body: RigidBody::Dynamic,
            model: Model::new("traffic_cone/model.glb#Mesh0/Primitive0"),
            hit_box: HitBox::new("traffic_cone/model.glb#Mesh1/Primitive0"),
            velocity: Velocity::default(),
            transform: TransformBundle::default(),
            material: Handle::default(),

            visibility: Visibility::default(),
            inherited_visibility: InheritedVisibility::default(),
            view_visibility: ViewVisibility::default(),
        }
    }
}
