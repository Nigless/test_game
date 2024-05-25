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
    crouching_state::CrouchingState, falling_state::FallingState, moving_state::MovingState,
    rising_state::RisingState, running_state::RunningState, Stats,
};

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct StandingState;

pub struct StandingStatePlugin;

impl Plugin for StandingStatePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<StandingState>()
            .add_systems(Update, switch);
    }
}

fn switch(
    mut commands: Commands,
    input: Res<Input>,
    entity_q: Query<(Entity, &Velocity, &CharacterBody), With<StandingState>>,
) {
    for (entity, velocity, character_body) in entity_q.iter() {
        let is_moving = velocity.linvel.xz().length() > 0.01;

        // standing: true,
        // moving: false,
        // rising: false,
        // falling: false,
        // crouching: false,

        if is_moving && character_body.is_grounded {
            commands
                .entity(entity)
                .remove::<StandingState>()
                .insert(MovingState);
        }

        if !character_body.is_grounded && velocity.linvel.y > 0.01 {
            commands
                .entity(entity)
                .remove::<StandingState>()
                .insert(RisingState);
        }

        if !character_body.is_grounded && velocity.linvel.y < -0.01 {
            commands
                .entity(entity)
                .remove::<StandingState>()
                .insert(FallingState);
        }

        if character_body.is_grounded && is_moving && input.running {
            commands
                .entity(entity)
                .remove::<StandingState>()
                .insert(RunningState);
        }

        if input.crouching {
            commands
                .entity(entity)
                .remove::<StandingState>()
                .insert(CrouchingState);
        }
    }
}
