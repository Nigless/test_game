use bevy::{
    app::{App, Plugin},
    ecs::{component::Component, reflect::ReflectComponent},
    prelude::*,
    transform::systems::propagate_transforms,
};
use bevy_rapier3d::{
    dynamics::Velocity,
    geometry::Collider,
    pipeline::QueryFilter,
    plugin::RapierContext,
    prelude::{ShapeCastOptions, ShapeCastStatus},
};

#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(Transform, Velocity)]
pub struct CharacterBody {
    pub skin_width: f32,
    pub max_slides: i32,
    pub min_cast_distance: f32,
    last_position: Vec3,
}

impl Default for CharacterBody {
    fn default() -> Self {
        Self {
            skin_width: 0.01,
            max_slides: 16,
            min_cast_distance: 2.0,
            last_position: Vec3::ZERO,
        }
    }
}
impl CharacterBody {
    pub fn skin_width(mut self, value: f32) -> Self {
        self.skin_width = value;
        self
    }
}

#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub struct CharacterBodySystems;

pub struct CharacterBodyPlugin;

impl Plugin for CharacterBodyPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CharacterBody>().add_systems(
            PreUpdate,
            (propagate_transforms, update)
                .chain()
                .in_set(CharacterBodySystems),
        );
    }
}

fn update(
    time: Res<Time>,
    rapier: Query<&RapierContext>,
    mut entity_q: Query<
        (
            Entity,
            &mut Velocity,
            &mut Transform,
            &GlobalTransform,
            &mut CharacterBody,
            &Collider,
        ),
        Without<RapierContext>,
    >,
) {
    let rapier = rapier.get_single().unwrap();

    for (entity, mut velocity, mut transform, global_transform, mut character_body, collider) in
        entity_q.iter_mut()
    {
        let filter = QueryFilter::new().exclude_collider(entity);

        let mut position = global_transform.translation();
        let mut remaining_distance = (velocity.linvel * time.delta_secs()).length();

        for _ in 0..character_body.max_slides {
            let vector_cast = velocity
                .linvel
                .clamp_length_min(character_body.min_cast_distance);

            let collision = rapier.cast_shape(
                position,
                global_transform.rotation(),
                vector_cast,
                collider,
                ShapeCastOptions::default(),
                filter,
            );

            let Some((_, collision)) = collision else {
                character_body.last_position = position;

                position += velocity.linvel.normalize_or_zero() * remaining_distance;

                break;
            };

            if let ShapeCastStatus::PenetratingOrWithinTargetDist = collision.status {
                position = character_body.last_position;
                continue;
            }

            character_body.last_position = position;

            let normal = collision.details.unwrap().normal1;

            let time_of_impact = collision.time_of_impact;

            let mut vector_to_collision = vector_cast * time_of_impact;

            vector_to_collision -= vector_to_collision
                * ((normal * character_body.skin_width).length()
                    / vector_to_collision.project_onto(normal).length());

            if vector_to_collision
                .normalize_or_zero()
                .dot(velocity.linvel.normalize_or_zero())
                < 0.0
            {
                vector_to_collision = vector_to_collision.project_onto(normal);
                remaining_distance += vector_to_collision.length();
            } else if vector_to_collision.length() > remaining_distance {
                character_body.last_position = position;

                position += velocity.linvel.normalize_or_zero() * remaining_distance;
                break;
            }

            position += vector_to_collision;

            remaining_distance -= vector_to_collision.length();

            let projected_velocity = velocity.linvel.reject_from(normal);

            remaining_distance *= projected_velocity.length() / velocity.linvel.length();

            velocity.linvel = projected_velocity;
        }

        transform.translation += position - global_transform.translation();
    }
}
