use std::f32::consts;

use bevy::{
    app::{App, Plugin},
    ecs::{component::Component, reflect::ReflectComponent},
    prelude::*,
};
use bevy_rapier3d::{
    dynamics::{GravityScale, Velocity},
    geometry::Collider,
    pipeline::QueryFilter,
    plugin::{RapierConfiguration, RapierContext},
};

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct CharacterBody {
    pub is_grounded: bool,
    pub collision: Option<Collision>,
}

impl Default for CharacterBody {
    fn default() -> Self {
        Self {
            is_grounded: false,
            collision: None,
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

#[derive(Reflect)]
pub struct Collision {
    pub velocity: Vec3,
    pub normal: Vec3,
}

fn collide_and_slide(
    rapier: &Res<RapierContext>,
    rotation: Quat,
    collider: &Collider,
    entity: Entity,
    velocity: Vec3,
    position: Vec3,
) -> Option<Collision> {
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

        if let Some(collision) = collide_and_slide(
            rapier,
            rotation,
            collider,
            entity,
            vector_slide,
            position + vector_to_surface,
        ) {
            return Some(Collision {
                velocity: vector_to_surface + collision.velocity,
                normal: collision.normal,
            });
        }

        return Some(Collision {
            velocity: vector_to_surface + vector_slide,
            normal: normal,
        });
    };

    return None;
}

fn update(
    time: Res<Time>,
    rapier: Res<RapierContext>,
    rapier_config: Res<RapierConfiguration>,
    mut entity_q: Query<(
        Entity,
        &mut Velocity,
        &mut Transform,
        &Collider,
        &mut CharacterBody,
    )>,
) {
    for (entity, mut velocity, mut transform, collider, mut character_body) in entity_q.iter_mut() {
        let rotation = transform.rotation;
        let position = transform.translation;

        character_body.is_grounded = false;

        rapier
            .cast_shape(
                position,
                rotation,
                rapier_config.gravity.normalize_or_zero() * 0.12,
                collider,
                1.0,
                true,
                QueryFilter::new().exclude_collider(entity),
            )
            .map_or(None, |(_, hit)| {
                hit.details
                    .map(|details| details.normal1.normalize_or_zero())
            })
            .map(|normal| {
                character_body.is_grounded =
                    rapier_config.gravity.angle_between(normal) > consts::PI * 0.75;
            });

        if let Some(collision) = collide_and_slide(
            &rapier,
            rotation,
            collider,
            entity,
            velocity.linvel,
            position,
        ) {
            velocity.linvel = collision.velocity;
            transform.translation += velocity.linvel;

            character_body.collision = Some(collision);
            continue;
        }

        character_body.collision = None;
        transform.translation += velocity.linvel;
    }
}
