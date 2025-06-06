use tracy_client::Client;

mod game;
mod library;

mod entities;
mod plugins;
mod scenes;
mod stores;
use game::game;

fn main() {
    #[cfg(debug_assertions)]
    let _client = Client::start();

    let mut game = game();

    game.run();
}
