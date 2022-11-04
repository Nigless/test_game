pub mod moving;
use super::utils::bnd_collider::BndCollider;
use super::utils::bnd_model::BndModel;
use super::utils::bnd_transform::BndTransform;
use crate::head::WithHead;
use bevy_rapier3d::prelude::Collider;

use bevy::prelude::*;
use bevy_rapier3d::prelude::LockedAxes;
use moving::Moving;

#[derive(Bundle)]
pub struct Player {
    head: WithHead,
    moving: Moving,
    locked_axes: LockedAxes,
    #[bundle]
    transform: BndTransform,
    #[bundle]
    collider: BndCollider,
    #[bundle]
    model: BndModel,
}

impl Player {
    pub fn new() -> Self {
        Self {
            head: WithHead,
            moving: Moving::default(),
            locked_axes: LockedAxes::ROTATION_LOCKED,
            transform: BndTransform::new(0.0, 2.0, 0.0),
            collider: BndCollider::new(Collider::cylinder(0.95, 0.2)),
            model: BndModel::new("robot/model.glb#Scene0"),
        }
    }
}
