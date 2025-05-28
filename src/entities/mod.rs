use bevy::app::Plugin;
use fireball::FireballPlugin;
use player::PlayerPlugin;
use traffic_cone::{TrafficCone, TrafficConePlugin};

pub mod block;
pub mod fireball;
pub mod player;
pub mod traffic_cone;

pub struct EntitiesPlugin;

impl Plugin for EntitiesPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins((FireballPlugin, TrafficConePlugin, PlayerPlugin));
    }
}
