use bevy::prelude::*;

use crate::stores::{app::AppStoresPlugin, game::GameStorePlugin, pause::PauseStorePlugin};

pub mod app;
pub mod game;
pub mod pause;

pub struct StoresPlugin;

impl Plugin for StoresPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((AppStoresPlugin, GameStorePlugin, PauseStorePlugin));
    }
}
