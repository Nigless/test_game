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
    crouching_state::CrouchingState, moving_state::MovingState, rising_state::RisingState,
    standing_state::StandingState, Stats,
};

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct FallingState;

pub struct FallingStatePlugin;

impl Plugin for FallingStatePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<FallingState>()
            .add_systems(Update, switch);
    }
}

fn switch(
    mut commands: Commands,
    input: Res<Input>,
    entity_q: Query<(Entity, &Velocity, &CharacterBody), With<FallingState>>,
) {
    for (entity, velocity, character_body) in entity_q.iter() {
        let is_moving = velocity.linvel.xz().length() > 0.01;

        if character_body.is_grounded {
            commands.entity(entity).remove::<FallingState>();
        }

        if !is_moving && !input.crouching {
            commands.entity(entity).insert(StandingState);
        }

        if is_moving {
            commands.entity(entity).insert(MovingState);
        }

        if !character_body.is_grounded && velocity.linvel.y > 0.01 {
            commands
                .entity(entity)
                .remove::<FallingState>()
                .insert(RisingState);
        }

        if input.crouching {
            commands.entity(entity).insert(CrouchingState);
        }
    }
}
