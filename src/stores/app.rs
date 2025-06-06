use bevy::prelude::*;

#[derive(Resource, Default)]
pub enum AppState {
    #[default]
    Loading,
    Game,
    Title,
}

pub struct AppStoresPlugin;

impl Plugin for AppStoresPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.init_resource::<AppState>();
    }
}
