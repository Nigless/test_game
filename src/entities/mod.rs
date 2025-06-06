use bevy::app::Plugin;
use fireball::FireballPlugin;
use gas_can::GasCanPlugin;
use player::PlayerPlugin;
use traffic_cone::{TrafficCone, TrafficConePlugin};

use crate::entities::{block::BlockPlugin, explosion::ExplosionPlugin};

pub mod block;
pub mod explosion;
pub mod fireball;
pub mod gas_can;
pub mod player;
pub mod traffic_cone;

pub struct EntitiesPlugin;

impl Plugin for EntitiesPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins((
            FireballPlugin,
            ExplosionPlugin,
            TrafficConePlugin,
            PlayerPlugin,
            GasCanPlugin,
            BlockPlugin,
        ));
    }
}
