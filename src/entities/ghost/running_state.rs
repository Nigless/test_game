use std::ops::Mul;

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
    character_body::{CharacterBody, CharacterBodySystems},
    control::{Control, Input},
};

use super::{
    crouching_falling_state::CrouchingFallingState,
    crouching_standing_state::CrouchingStandingState, falling_state::FallingState,
    moving_state::MovingState, standing_state::StandingState, GhostSystems, Stats,
    COLLIDER_CROUCHING_HALF_HEIGHT, COLLIDER_HALF_HEIGHT, COLLIDER_RADIUS,
};

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct RunningState;

pub struct RunningStatePlugin;

impl Plugin for RunningStatePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<RunningState>().add_systems(
            PreUpdate,
            (
                switch.in_set(GhostSystems::Switch),
                exit.in_set(GhostSystems::Exit),
                enter.in_set(GhostSystems::Enter),
                update.in_set(GhostSystems::Update),
            ),
        );
    }
}

fn enter(
    mut entity_q: Query<(&mut Collider, &mut Stats, &mut AnimationSequencer), Added<RunningState>>,
) {
    for (mut collider, mut stats, mut sequencer) in entity_q.iter_mut() {
        *collider = Collider::capsule_y(COLLIDER_HALF_HEIGHT, COLLIDER_RADIUS);

        stats.moving_speed = 0.04;
        stats.jumping_high = 0.032;
        stats.acceleration = 10.0;

        sequencer.set_transition("walking", 1.0, 500);
    }
}

fn exit(
    mut removed: RemovedComponents<RunningState>,
    mut entity_q: Query<&mut AnimationSequencer>,
) {
    for entity in removed.read() {
        let mut sequencer = entity_q.get_mut(entity).unwrap();

        sequencer.set_transition("walking", 0.0, 500);
    }
}

fn update(
    mut entity_q: Query<(&mut AnimationSequencer, &Velocity, &CharacterBody), With<RunningState>>,
) {
    for (mut sequencer, velocity, character_body) in entity_q.iter_mut() {
        let speed = velocity
            .linvel
            .reject_from(character_body.normal)
            .length()
            .mul(35.0);

        sequencer.set_speed("walking", speed);
    }
}

fn switch(
    mut commands: Commands,
    input: Res<Input>,
    mut entity_q: Query<
        (
            Entity,
            &Velocity,
            &CharacterBody,
            &mut Transform,
            &Children,
            Option<&Control>,
        ),
        With<RunningState>,
    >,
    mut head_q: Query<&mut Transform, Without<CharacterBody>>,
) {
    for (entity, velocity, character_body, mut transform, children, control) in entity_q.iter_mut()
    {
        let is_moving = velocity.linvel.reject_from(character_body.normal).length() > 0.01;

        if control.is_some() && input.crouching {
            if character_body.is_grounded {
                let head = *children.get(0).expect("character doesn't have head");

                let mut head_transform = head_q.get_mut(head).unwrap();

                head_transform.translation.y +=
                    COLLIDER_HALF_HEIGHT - COLLIDER_CROUCHING_HALF_HEIGHT;

                transform.translation.y -= COLLIDER_HALF_HEIGHT - COLLIDER_CROUCHING_HALF_HEIGHT;

                commands
                    .entity(entity)
                    .remove::<RunningState>()
                    .insert(CrouchingStandingState);

                continue;
            }

            commands
                .entity(entity)
                .remove::<RunningState>()
                .insert(CrouchingFallingState);

            continue;
        }

        if character_body.is_grounded {
            if is_moving {
                if control.is_some() && input.running {
                    continue;
                }

                commands
                    .entity(entity)
                    .remove::<RunningState>()
                    .insert(MovingState);

                continue;
            }

            commands
                .entity(entity)
                .remove::<RunningState>()
                .insert(StandingState);

            continue;
        }

        commands
            .entity(entity)
            .remove::<RunningState>()
            .insert(FallingState);
    }
}
