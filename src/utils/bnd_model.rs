use crate::model::WithModel;
use bevy::prelude::*;

#[derive(Bundle)]
pub struct BndModel {
    model: WithModel,
    visibility: Visibility,
    computed_visibility: ComputedVisibility,
}

impl BndModel {
    pub fn new(src: &str) -> Self {
        Self {
            model: WithModel::new(src),
            visibility: Default::default(),
            computed_visibility: Default::default(),
        }
    }
}
