use bevy::{
    app::{App, Plugin},
    ecs::{component::Component, reflect::ReflectComponent},
    prelude::*,
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
    pub cast_distance: f32,
    last_position: Vec3,
}

impl Default for CharacterBody {
    fn default() -> Self {
        Self {
            skin_width: 0.01,
            max_slides: 10,
            cast_distance: 2.0,
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
        app.register_type::<CharacterBody>()
            .add_systems(PreUpdate, update.in_set(CharacterBodySystems));
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

        for _ in 0..character_body.max_slides {
            let linear_velocity = velocity.linvel * time.delta_secs();

            let vector_cast = linear_velocity.clamp_length_min(character_body.cast_distance);

            let collision = rapier.cast_shape(
                global_transform.translation(),
                global_transform.rotation(),
                vector_cast,
                collider,
                ShapeCastOptions::default(),
                filter,
            );

            let Some((_, collision)) = collision else {
                character_body.last_position = transform.translation;

                transform.translation += linear_velocity;

                break;
            };

            if let ShapeCastStatus::PenetratingOrWithinTargetDist = collision.status {
                transform.translation = character_body.last_position;
                continue;
            }

            character_body.last_position = transform.translation;

            let normal = collision.details.unwrap().normal1;

            let time_of_impact = collision.time_of_impact;

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
                character_body.last_position = transform.translation;

                transform.translation += linear_velocity;
                break;
            }

            velocity.linvel = velocity.linvel.reject_from(normal);

            transform.translation += vector_to_collision;
        }
    }
}
