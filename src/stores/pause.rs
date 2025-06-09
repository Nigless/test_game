use bevy::prelude::*;

use crate::plugins::{input::PausingPressedEvent, prefab::PrefabsLoadedEvent};

#[derive(Resource, Default, PartialEq)]
pub enum PauseState {
    #[default]
    Pause,
    Playing,
}

impl PauseState {
    pub fn is_paused(state: Res<PauseState>) -> bool {
        *state == PauseState::Pause
    }

    pub fn is_not_paused(state: Res<PauseState>) -> bool {
        *state == PauseState::Playing
    }
}

pub struct PauseStorePlugin;

impl Plugin for PauseStorePlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.init_resource::<PauseState>()
            .add_observer(handle_pause)
            .add_observer(handle_loaded);
    }
}

fn handle_pause(_: Trigger<PausingPressedEvent>, mut state: ResMut<PauseState>) {
    *state = match *state {
        PauseState::Pause => PauseState::Playing,
        PauseState::Playing => PauseState::Pause,
    };
}

fn handle_loaded(_: Trigger<PrefabsLoadedEvent>, mut state: ResMut<PauseState>) {
    *state = PauseState::Playing
}
