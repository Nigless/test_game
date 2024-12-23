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

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct CharacterBody {
    pub normal: Vec3,
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
    let skin_width = 0.1;
    let min_move = 0.1;
    let max_depth = 10;

    for (entity, mut velocity, mut transform, mut character_body, collider) in entity_q.iter_mut() {
        character_body.normal = Vec3::ZERO;
        for _ in 0..max_depth {
            let vector_cast = velocity.linvel.normalize_or_zero()
                * (velocity.linvel.length() * time.delta_seconds() + skin_width);

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
                transform.translation +=
                    vector_cast.normalize_or_zero() * (vector_cast.length() - skin_width);

                break;
            }

            let (time_of_impact, details) = collision.unwrap();

            let normal = details.normal1.normalize_or_zero();

            character_body.normal = normal;

            let distance_to_collision = vector_cast.length() * time_of_impact - skin_width;

            let mut vector_to_collision = vector_cast.normalize_or_zero() * distance_to_collision;

            if distance_to_collision <= skin_width {
                vector_to_collision = Vec3::ZERO;
            }

            velocity.linvel = velocity.linvel.reject_from(normal);

            transform.translation += vector_to_collision;
        }
    }
}
