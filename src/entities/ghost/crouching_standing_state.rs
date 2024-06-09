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
use bevy_rapier3d::{
    dynamics::Velocity,
    geometry::Collider,
    pipeline::QueryFilter,
    plugin::{RapierConfiguration, RapierContext},
};

use crate::{
    animation_sequencer::AnimationSequencer,
    character_body::CharacterBody,
    control::{Control, Input},
};

use super::{
    crouching_falling_state::CrouchingFallingState, falling_state::FallingState,
    moving_state::MovingState, running_state::RunningState, standing_state::StandingState,
    GhostSystems, Stats, COLLIDER_CROUCHING_HALF_HEIGHT, COLLIDER_HALF_HEIGHT, COLLIDER_RADIUS,
};

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct CrouchingStandingState;

pub struct CrouchingStandingStatePlugin;

impl Plugin for CrouchingStandingStatePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CrouchingStandingState>().add_systems(
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

fn enter(mut entity_q: Query<(&mut Collider, &mut Stats), Added<CrouchingStandingState>>) {
    for (mut collider, mut stats) in entity_q.iter_mut() {
        *collider = Collider::capsule_y(COLLIDER_CROUCHING_HALF_HEIGHT, COLLIDER_RADIUS);

        stats.moving_speed = 0.015;
        stats.jumping_high = 0.02;
        stats.acceleration = 10.0;
    }
}

fn exit(
    mut removed: RemovedComponents<CrouchingStandingState>,
    mut entity_q: Query<&mut AnimationSequencer>,
) {
    for entity in removed.read() {
        let mut sequencer = entity_q.get_mut(entity).unwrap();

        sequencer.set_transition("walking", 0.0, 500);
    }
}

fn update(
    mut entity_q: Query<
        (&mut AnimationSequencer, &Velocity, &CharacterBody),
        With<CrouchingStandingState>,
    >,
) {
    for (mut sequencer, velocity, character_body) in entity_q.iter_mut() {
        let speed = velocity.linvel.reject_from(character_body.normal).length();

        if speed > 0.001 {
            sequencer.set_transition("walking", 2.0, 500)
        } else {
            sequencer.set_transition("walking", 0.0, 500);
        }

        sequencer.set_speed("walking", speed * 35.0);
    }
}

fn switch(
    mut commands: Commands,
    rapier: Res<RapierContext>,
    rapier_config: Res<RapierConfiguration>,
    input: Res<Input>,
    mut entity_q: Query<
        (
            Entity,
            &Velocity,
            &CharacterBody,
            &mut Transform,
            &Collider,
            &Children,
            Option<&Control>,
        ),
        With<CrouchingStandingState>,
    >,
    mut head_q: Query<&mut Transform, Without<CharacterBody>>,
) {
    for (entity, velocity, character_body, mut transform, collider, children, control) in
        entity_q.iter_mut()
    {
        let is_moving = velocity.linvel.reject_from(character_body.normal).length() > 0.01;

        let can_stand_up = rapier
            .cast_shape(
                transform.translation,
                transform.rotation,
                rapier_config.gravity.normalize_or_zero()
                    * -(COLLIDER_HALF_HEIGHT - COLLIDER_CROUCHING_HALF_HEIGHT)
                    * 2.0,
                &collider,
                1.0,
                true,
                QueryFilter::new().exclude_collider(entity),
            )
            .is_none();

        if (control.is_some() && input.crouching) || !can_stand_up {
            if character_body.is_grounded {
                continue;
            }

            commands
                .entity(entity)
                .remove::<CrouchingStandingState>()
                .insert(CrouchingFallingState);

            continue;
        }

        let head = *children.get(0).expect("character doesn't have head");

        let mut head_transform = head_q.get_mut(head).unwrap();

        head_transform.translation.y -= COLLIDER_HALF_HEIGHT - COLLIDER_CROUCHING_HALF_HEIGHT;

        if character_body.is_grounded {
            transform.translation.y += COLLIDER_HALF_HEIGHT - COLLIDER_CROUCHING_HALF_HEIGHT;

            if is_moving {
                if control.is_some() && input.running {
                    commands
                        .entity(entity)
                        .remove::<CrouchingStandingState>()
                        .insert(RunningState);

                    continue;
                }

                commands
                    .entity(entity)
                    .remove::<CrouchingStandingState>()
                    .insert(MovingState);

                continue;
            }

            commands
                .entity(entity)
                .remove::<CrouchingStandingState>()
                .insert(StandingState);

            continue;
        }

        commands
            .entity(entity)
            .remove::<CrouchingStandingState>()
            .insert(FallingState);
    }
}
