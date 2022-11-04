use crate::model::Model;
use bevy::prelude::*;

#[derive(Bundle)]
pub struct BndModel {
    model: Model,
    visibility: Visibility,
    computed_visibility: ComputedVisibility,
}

impl BndModel {
    pub fn new(src: &str) -> Self {
        Self {
            model: Model::new(src),
            visibility: Default::default(),
            computed_visibility: Default::default(),
        }
    }
}
