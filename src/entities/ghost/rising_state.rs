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
use bevy_rapier3d::{dynamics::Velocity, na::ComplexField};

use crate::{
    character_body::CharacterBody,
    control::{Control, Input},
};

use super::{
    crouching_state::CrouchingState, falling_state::FallingState, moving_state::MovingState,
    standing_state::StandingState, Stats,
};

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct RisingState;

pub struct RisingStatePlugin;

impl Plugin for RisingStatePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<RisingState>()
            .add_systems(Update, switch);
    }
}

fn switch(
    mut commands: Commands,
    input: Res<Input>,
    entity_q: Query<(Entity, &Velocity, &CharacterBody), With<RisingState>>,
) {
    for (entity, velocity, character_body) in entity_q.iter() {
        let is_moving = velocity.linvel.xz().length() > 0.01;

        if input.crouching {
            commands
                .entity(entity)
                .remove::<RisingState>()
                .insert(CrouchingState);

            continue;
        }

        if character_body.is_grounded {
            if is_moving {
                commands
                    .entity(entity)
                    .remove::<RisingState>()
                    .insert(MovingState);
                continue;
            }

            commands
                .entity(entity)
                .remove::<RisingState>()
                .insert(StandingState);

            continue;
        }

        if velocity.linvel.y < -0.01 {
            commands
                .entity(entity)
                .remove::<RisingState>()
                .insert(FallingState);
        }
    }
}
