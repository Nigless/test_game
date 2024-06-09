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
    crouching_standing_state::CrouchingStandingState, falling_state::FallingState,
    moving_state::MovingState, running_state::RunningState, standing_state::StandingState,
    GhostSystems, Stats, COLLIDER_CROUCHING_HALF_HEIGHT, COLLIDER_HALF_HEIGHT, COLLIDER_RADIUS,
};

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct CrouchingFallingState;

pub struct CrouchingFallingStatePlugin;

impl Plugin for CrouchingFallingStatePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CrouchingFallingState>().add_systems(
            PreUpdate,
            (
                switch.in_set(GhostSystems::Switch),
                enter.in_set(GhostSystems::Enter),
            ),
        );
    }
}

fn enter(mut entity_q: Query<(&mut Collider, &mut Stats), Added<CrouchingFallingState>>) {
    for (mut collider, mut stats) in entity_q.iter_mut() {
        *collider = Collider::capsule_y(COLLIDER_CROUCHING_HALF_HEIGHT, COLLIDER_RADIUS);

        stats.moving_speed = 0.015;
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
            &mut CharacterBody,
            &mut Transform,
            &Collider,
            &Children,
            Option<&Control>,
        ),
        With<CrouchingFallingState>,
    >,
    mut head_q: Query<&mut Transform, Without<CharacterBody>>,
) {
    for (entity, velocity, mut character_body, mut transform, collider, children, control) in
        entity_q.iter_mut()
    {
        let is_moving = velocity.linvel.reject_from(character_body.normal).length() > 0.01;

        let half_shape_distance = COLLIDER_HALF_HEIGHT - COLLIDER_CROUCHING_HALF_HEIGHT;

        let vector_cast =
            rapier_config.gravity.normalize_or_zero() * (half_shape_distance * 2.0 + 0.1);

        let max_distance = vector_cast.length();

        let filter = QueryFilter::new().exclude_collider(entity);

        let mut normal = Vec3::ZERO;

        let map = |time_to_impact: f32| vector_cast.length() * time_to_impact - 0.1;

        let distance_to_floor = rapier
            .cast_shape(
                transform.translation + velocity.linvel,
                transform.rotation,
                vector_cast,
                &collider,
                1.0,
                true,
                filter,
            )
            .map_or(max_distance, |(_, toi)| {
                normal = toi
                    .details
                    .map_or(normal, |d| d.normal1.normalize_or_zero());
                let value = map(toi.toi);
                value
            });

        let distance_to_ceiling = rapier
            .cast_shape(
                transform.translation + velocity.linvel,
                transform.rotation,
                vector_cast * -1.0,
                &collider,
                1.0,
                true,
                filter,
            )
            .map_or(max_distance, |(_, toi)| map(toi.toi));

        if (control.is_some() && input.crouching)
            || distance_to_floor + distance_to_ceiling < half_shape_distance * 2.0
        {
            if distance_to_floor <= 0.001 {
                character_body.normal = normal;
                commands
                    .entity(entity)
                    .remove::<CrouchingFallingState>()
                    .insert(CrouchingStandingState);

                continue;
            }

            continue;
        }

        if distance_to_floor < half_shape_distance || distance_to_ceiling < half_shape_distance {
            let vector_shift = if distance_to_floor < distance_to_ceiling {
                half_shape_distance - distance_to_floor
            } else {
                -(half_shape_distance - distance_to_ceiling)
            };

            let head = *children.get(0).expect("character doesn't have head");

            let mut head_transform = head_q.get_mut(head).unwrap();

            head_transform.translation.y -= vector_shift;

            transform.translation.y += vector_shift;

            character_body.normal = normal;
        }

        if distance_to_floor <= half_shape_distance {
            character_body.normal = normal;

            if is_moving {
                if control.is_some() && input.running {
                    commands
                        .entity(entity)
                        .remove::<CrouchingFallingState>()
                        .insert(RunningState);

                    continue;
                }

                commands
                    .entity(entity)
                    .remove::<CrouchingFallingState>()
                    .insert(MovingState);

                continue;
            }

            commands
                .entity(entity)
                .remove::<CrouchingFallingState>()
                .insert(StandingState);

            continue;
        }

        commands
            .entity(entity)
            .remove::<CrouchingFallingState>()
            .insert(FallingState);
    }
}
