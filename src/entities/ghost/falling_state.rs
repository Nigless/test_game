use bevy::{
    app::{App, Plugin},
    ecs::{
        component::Component,
        query::With,
        reflect::ReflectComponent,
        system::{Query, Res},
    },
    prelude::*,
    reflect::Reflect,
};
use bevy_rapier3d::{dynamics::Velocity, geometry::Collider};

use crate::{
    animation_sequencer::AnimationSequencer,
    character_body::CharacterBody,
    control::{Control, Input},
};

use super::{
    crouching_falling_state::CrouchingFallingState,
    crouching_standing_state::CrouchingStandingState, moving_state::MovingState,
    running_state::RunningState, standing_state::StandingState, GhostSystems, Stats,
    COLLIDER_HALF_HEIGHT, COLLIDER_RADIUS,
};

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct FallingState;

pub struct FallingStatePlugin;

impl Plugin for FallingStatePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<FallingState>().add_systems(
            PreUpdate,
            (
                switch.in_set(GhostSystems::Switch),
                exit.in_set(GhostSystems::Exit),
                enter.in_set(GhostSystems::Enter),
            ),
        );
    }
}

fn enter(
    mut entity_q: Query<(&mut Collider, &mut Stats, &mut AnimationSequencer), Added<FallingState>>,
) {
    for (mut collider, mut stats, _sequencer) in entity_q.iter_mut() {
        *collider = Collider::capsule_y(COLLIDER_HALF_HEIGHT, COLLIDER_RADIUS);

        stats.moving_speed = 0.015;
    }
}

fn exit(
    mut entity_q: Query<&mut AnimationSequencer>,
    mut removed: RemovedComponents<FallingState>,
) {
    for entity in removed.read() {
        let _sequencer = entity_q.get_mut(entity).unwrap();
    }
}

fn switch(
    mut commands: Commands,
    input: Res<Input>,
    entity_q: Query<(Entity, &Velocity, &CharacterBody, Option<&Control>), With<FallingState>>,
) {
    for (entity, velocity, character_body, control) in entity_q.iter() {
        let is_moving = velocity.linvel.reject_from(character_body.normal).length() > 0.01;

        if control.is_some() && input.crouching {
            if character_body.is_grounded {
                commands
                    .entity(entity)
                    .remove::<FallingState>()
                    .insert(CrouchingStandingState);

                continue;
            }

            commands
                .entity(entity)
                .remove::<FallingState>()
                .insert(CrouchingFallingState);

            continue;
        }

        if character_body.is_grounded {
            if is_moving {
                if control.is_some() && input.running {
                    commands
                        .entity(entity)
                        .remove::<FallingState>()
                        .insert(RunningState);

                    continue;
                }

                commands
                    .entity(entity)
                    .remove::<FallingState>()
                    .insert(MovingState);

                continue;
            }

            commands
                .entity(entity)
                .remove::<FallingState>()
                .insert(StandingState);

            continue;
        }
    }
}
