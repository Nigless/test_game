use std::f32::consts;

use bevy::{
    app::{App, Plugin},
    ecs::{component::Component, reflect::ReflectComponent},
    prelude::*,
};
use bevy_rapier3d::{
    dynamics::Velocity,
    geometry::Collider,
    pipeline::QueryFilter,
    plugin::{RapierConfiguration, RapierContext},
};

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct CharacterBody {
    pub skin_width: f32,
    pub max_slides: i32,
    pub cast_distance: f32,
}

impl Default for CharacterBody {
    fn default() -> Self {
        Self {
            skin_width: 0.1,
            max_slides: 10,
            cast_distance: 10.0,
        }
    }
}

pub struct CharacterBodyPlugin;

impl Plugin for CharacterBodyPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CharacterBody>()
            .add_systems(Update, update);
    }
}

fn update(
    time: Res<Time>,
    rapier: Res<RapierContext>,
    mut entity_q: Query<(
        Entity,
        &mut Velocity,
        &mut Transform,
        &mut CharacterBody,
        &Collider,
    )>,
) {
    for (entity, mut velocity, mut transform, mut character_body, collider) in entity_q.iter_mut() {
        for _ in 0..character_body.max_slides {
            let linear_velocity = velocity.linvel * time.delta_seconds();

            let vector_cast = linear_velocity
                .clamp_length_min(character_body.skin_width * character_body.cast_distance);

            let collision = rapier
                .cast_shape(
                    transform.translation,
                    transform.rotation,
                    vector_cast,
                    collider,
                    1.0,
                    true,
                    QueryFilter::new().exclude_collider(entity),
                )
                .map_or(None, |(_, hit)| {
                    hit.details.map(|details| (hit.toi, details))
                });

            if collision.is_none() {
                transform.translation += linear_velocity;

                break;
            }

            let (time_of_impact, details) = collision.unwrap();

            let normal = details.normal1;

            let mut vector_to_collision = vector_cast * time_of_impact;

            vector_to_collision -= vector_to_collision
                * ((normal * character_body.skin_width).length()
                    / vector_to_collision.project_onto(normal).length());

            if vector_to_collision
                .normalize_or_zero()
                .dot(linear_velocity.normalize_or_zero())
                < 0.0
            {
                vector_to_collision = vector_to_collision.project_onto(normal);
            } else if vector_to_collision.length() > linear_velocity.length() {
                transform.translation += linear_velocity;
                break;
            }

            velocity.linvel = velocity.linvel.reject_from(normal);

            transform.translation += vector_to_collision;
        }
    }
}
