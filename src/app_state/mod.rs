use bevy::{
    app::{App, Plugin, Startup},
    asset::AssetServer,
    ecs::{
        system::{Commands, Res},
        world::FromWorld,
    },
    state::{
        app::AppExtStates,
        commands::{self, CommandsStatesExt},
        state::States,
    },
};
use game::GameStatePlugin;
use title::TitleStatePlugin;

mod game;
mod title;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]
    Loading,
    Game,
    Title,
}

pub struct AppStatePlugin;

impl Plugin for AppStatePlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.init_state::<AppState>()
            .add_plugins(GameStatePlugin(AppState::Game))
            .add_plugins(TitleStatePlugin(AppState::Title))
            .add_systems(Startup, startup);
    }
}

fn startup(mut commands: Commands) {
    commands.set_state(AppState::Game);
}
