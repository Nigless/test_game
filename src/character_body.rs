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
    pub is_grounded: bool,
    pub normal: Vec3,
}

#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub enum CharacterBodySystems {
    Update,
}

pub struct CharacterBodyPlugin;

impl Plugin for CharacterBodyPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CharacterBody>()
            .configure_sets(PreUpdate, CharacterBodySystems::Update)
            .add_systems(PreUpdate, update.in_set(CharacterBodySystems::Update));
    }
}

fn collide_and_slide(
    rapier: &Res<RapierContext>,
    rotation: Quat,
    collider: &Collider,
    entity: Entity,
    velocity: Vec3,
    position: Vec3,
) -> Vec3 {
    let skin_width = 0.1;

    let vector_cast = velocity.normalize_or_zero() * (velocity.length() + skin_width);
    let collision = rapier
        .cast_shape(
            position,
            rotation,
            vector_cast,
            collider,
            1.0,
            true,
            QueryFilter::new().exclude_collider(entity),
        )
        .map_or(None, |(_, hit)| {
            hit.details.map(|details| (hit.toi, details))
        });

    if let Some((time_of_impact, details)) = collision {
        let normal = details.normal1.normalize_or_zero();

        let mut vector_to_surface =
            vector_cast.normalize_or_zero() * (vector_cast.length() * time_of_impact - skin_width);

        if vector_to_surface.length() <= skin_width {
            vector_to_surface = Vec3::ZERO;
        }

        let vector_slide = velocity.reject_from(normal);

        return vector_to_surface
            + collide_and_slide(
                rapier,
                rotation,
                collider,
                entity,
                vector_slide,
                position + vector_to_surface,
            );
    };

    return velocity;
}

fn update(
    rapier: Res<RapierContext>,
    rapier_config: Res<RapierConfiguration>,
    mut entity_q: Query<(
        Entity,
        &mut Velocity,
        &mut Transform,
        &mut CharacterBody,
        &Collider,
    )>,
) {
    for (entity, mut velocity, mut transform, mut character_body, collider) in entity_q.iter_mut() {
        velocity.linvel = collide_and_slide(
            &rapier,
            transform.rotation,
            collider,
            entity,
            velocity.linvel,
            transform.translation,
        );
        transform.translation += velocity.linvel;

        character_body.is_grounded = false;
        character_body.normal = Vec3::ZERO;

        rapier
            .cast_shape(
                transform.translation,
                transform.rotation,
                rapier_config.gravity.normalize_or_zero() * 0.14,
                collider,
                1.0,
                true,
                QueryFilter::new().exclude_collider(entity),
            )
            .map_or(None, |(_, hit)| hit.details)
            .map(|details| details.normal1.normalize_or_zero())
            .map(|normal| {
                character_body.normal = normal;
                character_body.is_grounded =
                    rapier_config.gravity.angle_between(normal) > consts::PI * 0.70;
            });
    }
}
