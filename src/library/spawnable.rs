use bevy::prelude::{Bundle, Commands, Entity, EntityCommands};

pub trait Spawnable {
    fn spawn(&self, commands: &mut Commands) -> Entity;
}
