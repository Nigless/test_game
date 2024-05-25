use bevy::{
    app::{App, Plugin, Update},
    ecs::{
        component::Component,
        query::With,
        reflect::ReflectComponent,
        schedule::IntoSystemConfigs,
        system::{Query, Res},
    },
    prelude::*,
    reflect::Reflect,
    time::Time,
    transform::components::Transform,
};
use bevy_rapier3d::dynamics::Velocity;

use crate::{
    character_body::CharacterBody,
    control::{Control, Input},
};

use super::{
    falling_state::FallingState, moving_state::MovingState, rising_state::RisingState,
    standing_state::StandingState, Stats,
};

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct CrouchingState;

pub struct CrouchingStatePlugin;

impl Plugin for CrouchingStatePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CrouchingState>()
            .add_systems(Update, switch);
    }
}

fn switch(
    mut commands: Commands,
    input: Res<Input>,
    entity_q: Query<(Entity, &Velocity, &CharacterBody), With<CrouchingState>>,
) {
    for (entity, velocity, character_body) in entity_q.iter() {
        let is_moving = velocity.linvel.xz().length() > 0.01;

        if !input.crouching {
            commands.entity(entity).remove::<CrouchingState>();
        }

        if character_body.is_grounded && !is_moving {
            commands.entity(entity).insert(StandingState);
        }

        if character_body.is_grounded && is_moving {
            commands.entity(entity).insert(MovingState);
        }

        if !character_body.is_grounded && velocity.linvel.y > 0.01 {
            commands.entity(entity).insert(RisingState);
        }

        if !character_body.is_grounded && velocity.linvel.y < -0.01 {
            commands.entity(entity).insert(FallingState);
        }
    }
}
