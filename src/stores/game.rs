use bevy::prelude::*;

#[derive(Event)]
pub struct PlayerDeadEvent;

#[derive(Resource, Default)]
pub enum GameState {
    #[default]
    Playing,
    Death,
    Menu,
}

pub struct GameStorePlugin;

impl Plugin for GameStorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameState>()
            .add_observer(handle_player_dead);
    }
}

fn handle_player_dead(_: Trigger<PlayerDeadEvent>, mut state: ResMut<GameState>) {
    *state = match *state {
        GameState::Playing => GameState::Death,
        GameState::Death => todo!(),
        GameState::Menu => todo!(),
    }
}
