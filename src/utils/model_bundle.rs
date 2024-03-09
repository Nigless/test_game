use crate::model::WithModel;
use bevy::prelude::*;

#[derive(Bundle)]
pub struct ModelBundle {
    model: WithModel,
    visibility: Visibility,
    computed_visibility: InheritedVisibility,
}

impl ModelBundle {
    pub fn new(src: &str) -> Self {
        Self {
            model: WithModel::new(src),
            visibility: Default::default(),
            computed_visibility: Default::default(),
        }
    }
}
