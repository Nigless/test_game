use bevy::prelude::*;
use bevy_inspector_egui::egui::util::cache;
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
    pub fixed_update: bool,
    exclude: Option<Entity>,
}

impl ShapeCaster {
    pub fn new(collider: Collider, direction: Vec3) -> Self {
        Self {
            collider,
            direction,
            result: None,
            fixed_update: false,
            exclude: None,
        }
    }

    pub fn fixed_update(mut self) -> Self {
        self.fixed_update = true;
        self
    }

    pub fn exclude(mut self, entity: Entity) -> Self {
        self.exclude = Some(entity);
        self
    }
}

pub struct ShapeCasterPlugin;

impl Plugin for ShapeCasterPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ShapeCaster>()
            .add_systems(FixedPreUpdate, fixed_update.in_set(ShapeCasterSystems))
            .add_systems(PreUpdate, update.in_set(ShapeCasterSystems));
    }
}

fn fixed_update(
    rapier: Single<&RapierContext>,
    mut entity_q: Query<(&mut ShapeCaster, &GlobalTransform), Without<RapierContext>>,
) {
    for entity in entity_q.iter_mut() {
        if !entity.0.fixed_update {
            continue;
        }

        update_entity(&rapier, entity);
    }
}

fn update(
    rapier: Single<&RapierContext>,
    mut entity_q: Query<(&mut ShapeCaster, &GlobalTransform), Without<RapierContext>>,
) {
    for entity in entity_q.iter_mut() {
        if entity.0.fixed_update {
            continue;
        }

        update_entity(&rapier, entity);
    }
}

fn update_entity(
    rapier: &Single<&RapierContext>,
    (mut shape_caster, transform): (Mut<'_, ShapeCaster>, &GlobalTransform),
) {
    let mut filter = QueryFilter::default();

    filter.exclude_collider = shape_caster.exclude;

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

        return;
    }

    shape_caster.result = None
}
