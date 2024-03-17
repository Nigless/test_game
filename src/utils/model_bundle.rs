use crate::model::Model;
use bevy::prelude::*;

#[derive(Bundle)]
pub struct ModelBundle {
    model: Model,
    visibility: Visibility,
    computed_visibility: InheritedVisibility,
}

impl ModelBundle {
    pub fn new(src: &str) -> Self {
        Self {
            model: Model::new(src),
            visibility: Default::default(),
            computed_visibility: Default::default(),
        }
    }
}
