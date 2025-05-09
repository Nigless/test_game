use bevy::prelude::{Bundle, Commands, Entity, EntityCommands};

pub trait Spawnable {
    fn spawn<'a>(&self, commands: &'a mut Commands) -> EntityCommands<'a>;
}
