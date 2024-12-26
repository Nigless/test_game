use bevy::{prelude::*, utils::HashMap};
use bevy_rapier3d::{
    plugin::RapierContext,
    prelude::{Collider, QueryFilter, QueryFilterFlags},
};

#[derive(Reflect)]
pub struct CastResult {
    pub distance: f32,
    pub normal: Vec3,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ShapeCaster {
    pub direction: Vec3,
    pub result: Option<CastResult>,
}

impl ShapeCaster {
    pub fn new(direction: Vec3) -> Self {
        Self {
            direction,
            result: None,
        }
    }
}

pub struct ShapeCasterPlugin;

impl Plugin for ShapeCasterPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ShapeCaster>()
            .add_systems(PreUpdate, update);
    }
}

fn update(
    rapier: Res<RapierContext>,
    mut entity_q: Query<(&mut ShapeCaster, &Collider, &GlobalTransform)>,
) {
    let filter = QueryFilter::from(QueryFilterFlags::EXCLUDE_SENSORS);

    for (mut shape_caster, collider, transform) in entity_q.iter_mut() {
        if let Some((time_of_impact, normal)) = rapier
            .cast_shape(
                transform.translation(),
                transform.to_scale_rotation_translation().1,
                shape_caster.direction,
                collider,
                1.0,
                true,
                filter,
            )
            .map_or(None, |(_, hit)| {
                hit.details.map(|details| (hit.toi, details.normal1))
            })
        {
            shape_caster.result = Some(CastResult {
                distance: shape_caster.direction.length() * time_of_impact,
                normal,
            });

            continue;
        }

        shape_caster.result = None
    }
}
