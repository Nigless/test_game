use bevy::prelude::*;
use bevy_rapier3d::{
    plugin::RapierContext,
    prelude::{Collider, QueryFilter, ShapeCastOptions},
};

#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub struct ShapeCasterSystems;

#[derive(Reflect)]
pub struct CastResult {
    pub distance: f32,
    pub normal: Vec3,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(Transform)]
pub struct ShapeCaster {
    #[reflect(ignore)]
    pub collider: Collider,
    pub direction: Vec3,
    pub result: Option<CastResult>,
    pub exclude_parent: bool,
}

impl ShapeCaster {
    pub fn new(collider: Collider, direction: Vec3) -> Self {
        Self {
            collider,
            direction,
            result: None,
            exclude_parent: false,
        }
    }

    pub fn exclude_parent(mut self) -> Self {
        self.exclude_parent = true;
        self
    }
}

pub struct ShapeCasterPlugin;

impl Plugin for ShapeCasterPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ShapeCaster>()
            .add_systems(PreUpdate, update.in_set(ShapeCasterSystems));
    }
}

fn update(
    rapier_q: Query<&RapierContext>,
    mut entity_q: Query<
        (&mut ShapeCaster, &GlobalTransform, Option<&Parent>),
        Without<RapierContext>,
    >,
) {
    let rapier = rapier_q.get_single().unwrap();

    for (mut shape_caster, transform, parent) in entity_q.iter_mut() {
        let mut filter = QueryFilter::default();

        if let Some(parent) = parent {
            if shape_caster.exclude_parent {
                filter = filter.exclude_collider(parent.get());
            }
        }

        if let Some((time_of_impact, normal)) = rapier
            .cast_shape(
                transform.translation(),
                transform.rotation(),
                shape_caster.direction,
                &shape_caster.collider,
                ShapeCastOptions::default(),
                filter,
            )
            .map_or(None, |(_, hit)| {
                hit.details
                    .map(|details| (hit.time_of_impact, details.normal1))
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
