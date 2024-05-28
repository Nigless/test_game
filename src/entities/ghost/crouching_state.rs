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
    transform::{commands, components::Transform},
};
use bevy_rapier3d::{dynamics::Velocity, geometry::Collider, parry::either::IntoEither};

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
            .add_systems(Update, switch)
            .add_systems(Update, enter)
            .add_systems(Update, exit);
    }
}

fn enter(mut commands: Commands, entity_q: Query<Entity, Added<CrouchingState>>) {
    for entity in entity_q.iter() {
        commands
            .entity(entity)
            .insert(Collider::capsule_y(0.2, 0.2));
    }
}

fn exit(
    mut commands: Commands,
    mut removed_q: RemovedComponents<CrouchingState>,
    mut entity_q: Query<(&mut Transform, &CharacterBody), Without<CrouchingState>>,
) {
    for entity in removed_q.read() {
        if entity_q.get(entity).is_err() {
            continue;
        }

        let (mut transform, character_body) = entity_q.get_mut(entity).unwrap();

        if character_body.is_grounded {
            transform.translation += Vec3::Y * 0.7
        }

        commands
            .entity(entity)
            .insert(Collider::capsule_y(0.9, 0.2));
    }
}

fn switch(
    mut commands: Commands,
    input: Res<Input>,
    entity_q: Query<(Entity, &Velocity, &CharacterBody), With<CrouchingState>>,
) {
    for (entity, velocity, character_body) in entity_q.iter() {
        let is_moving = velocity.linvel.xz().length() > 0.01;

        if input.crouching {
            continue;
        }

        if character_body.is_grounded {
            if is_moving {
                commands
                    .entity(entity)
                    .remove::<CrouchingState>()
                    .insert(MovingState);

                continue;
            }

            commands
                .entity(entity)
                .remove::<CrouchingState>()
                .insert(StandingState);

            continue;
        }

        if velocity.linvel.y > 0.01 {
            commands
                .entity(entity)
                .remove::<CrouchingState>()
                .insert(RisingState);

            continue;
        }

        if velocity.linvel.y < -0.01 {
            commands
                .entity(entity)
                .remove::<CrouchingState>()
                .insert(FallingState);
        }
    }
}
