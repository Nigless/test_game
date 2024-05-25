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
    crouching_state::CrouchingState, falling_state::FallingState, rising_state::RisingState,
    standing_state::StandingState, Stats,
};

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct RunningState;

pub struct RunningStatePlugin;

impl Plugin for RunningStatePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<RunningState>()
            .add_systems(Update, switch);
    }
}

fn switch(
    mut commands: Commands,
    input: Res<Input>,
    entity_q: Query<(Entity, &Velocity, &CharacterBody), With<RunningState>>,
) {
    for (entity, velocity, character_body) in entity_q.iter() {
        let is_moving = velocity.linvel.xz().length() > 0.01;

        if !is_moving && character_body.is_grounded && !input.crouching {
            commands
                .entity(entity)
                .remove::<RunningState>()
                .insert(StandingState);
        }

        if velocity.linvel.y > 0.01 && !character_body.is_grounded {
            commands
                .entity(entity)
                .remove::<RunningState>()
                .insert(RisingState);
        }

        if velocity.linvel.y < -0.01 && !character_body.is_grounded {
            commands
                .entity(entity)
                .remove::<RunningState>()
                .insert(FallingState);
        }

        if input.crouching {
            commands
                .entity(entity)
                .remove::<RunningState>()
                .insert(CrouchingState);
        }

        // standing: true,
        // moving: false,
        // rising: false,
        // falling: false,
        // crouching: false,
    }
}
