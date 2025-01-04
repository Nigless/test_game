use bevy::{ecs::entity, prelude::*};
use bevy_rapier3d::{
    plugin::RapierContext,
    prelude::{Collider, QueryFilter, ShapeCastOptions},
};

use crate::Debugging;

#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub struct RayCasterSystems;

#[derive(Reflect)]
pub struct CastResult {
    pub distance: f32,
    pub normal: Vec3,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(Transform)]
pub struct RayCaster {
    pub direction: Vec3,
    pub result: Option<CastResult>,
    pub fixed_update: bool,
    exclude: Option<Entity>,
}

impl RayCaster {
    pub fn new(direction: Vec3) -> Self {
        Self {
            direction,
            result: None,
            fixed_update: false,
            exclude: None,
        }
    }

    pub fn exclude(mut self, entity: Entity) -> Self {
        self.exclude = Some(entity);
        self
    }

    pub fn fixed_update(mut self) -> Self {
        self.fixed_update = true;
        self
    }
}

pub struct RayCasterPlugin;

impl Plugin for RayCasterPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<RayCaster>()
            .add_systems(FixedPreUpdate, fixed_update.in_set(RayCasterSystems))
            .add_systems(PreUpdate, update.in_set(RayCasterSystems));
    }
}

fn fixed_update(
    debugging: Res<Debugging>,
    mut gizmos: Gizmos,
    rapier: Single<&RapierContext>,
    mut entity_q: Query<(&mut RayCaster, &GlobalTransform), Without<RapierContext>>,
) {
    for entity in entity_q.iter_mut() {
        if !entity.0.fixed_update {
            continue;
        }

        update_entity(&debugging, &mut gizmos, &rapier, entity);
    }
}

fn update(
    debugging: Res<Debugging>,
    mut gizmos: Gizmos,
    rapier: Single<&RapierContext>,
    mut entity_q: Query<(&mut RayCaster, &GlobalTransform), Without<RapierContext>>,
) {
    for entity in entity_q.iter_mut() {
        if entity.0.fixed_update {
            continue;
        }

        update_entity(&debugging, &mut gizmos, &rapier, entity);
    }
}

fn update_entity(
    debugging: &Res<Debugging>,
    gizmos: &mut Gizmos,
    rapier: &Single<&RapierContext>,
    (mut shape_caster, transform): (Mut<'_, RayCaster>, &GlobalTransform),
) {
    let mut filter = QueryFilter::default();

    filter.exclude_collider = shape_caster.exclude;

    if let Some((time_of_impact, normal)) = rapier
        .cast_ray_and_get_normal(
            transform.translation(),
            transform.rotation() * shape_caster.direction,
            1.0,
            false,
            filter,
        )
        .and_then(|(_, hit)| Some((hit.time_of_impact, hit.normal)))
    {
        shape_caster.result = Some(CastResult {
            distance: shape_caster.direction.length() * time_of_impact,
            normal,
        });

        if !debugging.enable {
            return;
        }

        gizmos.ray(
            transform.translation(),
            transform.rotation() * shape_caster.direction * time_of_impact,
            Color::linear_rgb(1.0, 0.0, 0.0),
        );

        gizmos
            .circle(
                Isometry3d::new(
                    transform.translation()
                        + transform.rotation() * shape_caster.direction * time_of_impact
                        + normal * 0.001,
                    Quat::from_rotation_arc(Vec3::Z, normal),
                ),
                0.1,
                Color::linear_rgb(1.0, 0.0, 0.0),
            )
            .resolution(16);

        return;
    }

    if !debugging.enable {
        return;
    }

    gizmos.ray(
        transform.translation(),
        transform.rotation() * shape_caster.direction,
        Color::linear_rgb(0.0, 0.0, 1.0),
    );

    shape_caster.result = None
}
