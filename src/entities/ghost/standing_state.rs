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
    transform::components::Transform,
};
use bevy_rapier3d::{dynamics::Velocity, geometry::Collider};

use crate::{
    animation_sequencer::AnimationSequencer,
    character_body::CharacterBody,
    control::{Control, Input},
};

use super::{
    crouching_falling_state::CrouchingFallingState,
    crouching_standing_state::CrouchingStandingState, falling_state::FallingState,
    moving_state::MovingState, running_state::RunningState, GhostSystems, Stats,
    COLLIDER_CROUCHING_HALF_HEIGHT, COLLIDER_HALF_HEIGHT, COLLIDER_RADIUS,
};

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct StandingState;

pub struct StandingStatePlugin;

impl Plugin for StandingStatePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<StandingState>().add_systems(
            PreUpdate,
            (
                switch.in_set(GhostSystems::Switch),
                exit.in_set(GhostSystems::Exit),
                enter.in_set(GhostSystems::Enter),
            ),
        );
    }
}

fn enter(mut entity_q: Query<(&mut Collider, &mut Stats), Added<StandingState>>) {
    for (mut collider, mut stats) in entity_q.iter_mut() {
        *collider = Collider::capsule_y(COLLIDER_HALF_HEIGHT, COLLIDER_RADIUS);

        stats.moving_speed = 0.025;
        stats.jumping_high = 0.032;
        stats.acceleration = 10.0;
    }
}

fn exit(
    mut removed: RemovedComponents<StandingState>,
    mut entity_q: Query<&mut AnimationSequencer>,
) {
    for entity in removed.read() {
        let _sequencer = entity_q.get_mut(entity).unwrap();
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
        With<StandingState>,
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
                    .remove::<StandingState>()
                    .insert(CrouchingStandingState);

                continue;
            }

            commands
                .entity(entity)
                .remove::<StandingState>()
                .insert(CrouchingFallingState);

            continue;
        }

        if character_body.is_grounded {
            if is_moving {
                if control.is_some() && input.running {
                    commands
                        .entity(entity)
                        .remove::<StandingState>()
                        .insert(RunningState);

                    continue;
                }

                commands
                    .entity(entity)
                    .remove::<StandingState>()
                    .insert(MovingState);

                continue;
            }

            continue;
        }

        commands
            .entity(entity)
            .remove::<StandingState>()
            .insert(FallingState);
    }
}
