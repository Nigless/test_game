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
    gravity: Vec3,
    mut velocity: Vec3,
    position: Vec3,
) -> Option<Collision> {
    let skin_width = 0.1;

    let mut vector_cast = velocity + gravity;
    vector_cast = vector_cast.normalize_or_zero() * (vector_cast.length() + skin_width);
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

        if gravity.angle_between(normal) < consts::PI * 0.75 {
            velocity = velocity + gravity - vector_to_surface
        }

        let vector_slide = velocity.reject_from(normal);

        if let Some(collision) = collide_and_slide(
            rapier,
            rotation,
            collider,
            entity,
            Vec3::ZERO,
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
        Option<&GravityScale>,
    )>,
) {
    for (entity, mut velocity, mut transform, collider, mut character_body, gravity) in
        entity_q.iter_mut()
    {
        let rotation = transform.rotation;
        let position = transform.translation;
        let vector_gravity = rapier_config.gravity
            * gravity.map(|v| v.0).unwrap_or(1.0)
            * time.delta_seconds().powi(2);

        if let Some(collision) = collide_and_slide(
            &rapier,
            rotation,
            collider,
            entity,
            vector_gravity,
            velocity.linvel,
            position,
        ) {
            velocity.linvel = collision.velocity;
            transform.translation += velocity.linvel;

            character_body.is_grounded =
                collision.normal.angle_between(vector_gravity) > consts::PI * 0.75;

            character_body.collision = Some(collision);
            continue;
        }
        character_body.is_grounded = false;
        character_body.collision = None;

        velocity.linvel += vector_gravity;
        transform.translation += velocity.linvel;
    }
}
