use bevy::{
    ecs::{
        component::{ComponentHooks, ComponentId, StorageType},
        world::DeferredWorld,
    },
    prelude::*,
};
use bevy_rapier3d::{
    plugin::{RapierConfiguration, RapierContext},
    prelude::{Group, QueryFilter, SolverGroups},
};

use crate::plugins::settings::Settings;

#[derive(Reflect)]
pub struct CasterResult {
    pub entity: Entity,
    pub distance: f32,
    pub normal: Vec3,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct RayCasterFixed;

#[derive(Reflect, Component)]
#[component(on_add = insert_tag_for_fixed)]
#[reflect(Component)]
#[require(Transform)]
pub struct RayCaster {
    pub direction: Vec3,
    pub result: Option<CasterResult>,
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

fn insert_tag_for_fixed(mut world: DeferredWorld<'_>, entity: Entity, _component_id: ComponentId) {
    let fixed_update = world.get::<RayCaster>(entity).unwrap().fixed_update;

    if fixed_update {
        world.commands().entity(entity).insert(RayCasterFixed);
    }
}

pub struct RayCasterPlugin;

impl Plugin for RayCasterPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<RayCaster>()
            .register_type::<RayCasterFixed>()
            .add_systems(FixedFirst, update::<RayCasterFixed>)
            .add_systems(First, update::<RayCaster>);
    }
}

fn update<T: Component>(
    mut gizmos: Gizmos,
    rapier: Single<&RapierContext>,
    mut entity_q: Query<(&mut RayCaster, &GlobalTransform), (Without<RapierContext>, With<T>)>,
    settings: Res<Settings>,
) {
    let mut filter = QueryFilter::default().exclude_sensors();

    for (mut caster, transform) in entity_q.iter_mut() {
        filter.exclude_collider = caster.exclude;

        if let Some((entity, time_of_impact, normal)) = rapier
            .cast_ray_and_get_normal(
                transform.translation(),
                transform.rotation() * caster.direction,
                1.0,
                false,
                filter,
            )
            .and_then(|(entity, hit)| Some((entity, hit.time_of_impact, hit.normal)))
        {
            caster.result = Some(CasterResult {
                entity: rapier.collider_parent(entity).unwrap_or(entity),
                distance: caster.direction.length() * time_of_impact,
                normal,
            });

            if !settings.dev_settings.show_ray_casters {
                continue;
            }

            gizmos.ray(
                transform.translation(),
                transform.rotation() * caster.direction * time_of_impact,
                Color::linear_rgb(1.0, 0.0, 0.0),
            );

            gizmos
                .circle(
                    Isometry3d::new(
                        transform.translation()
                            + transform.rotation() * caster.direction * time_of_impact
                            + normal * 0.001,
                        Quat::from_rotation_arc(Vec3::Z, normal),
                    ),
                    0.1,
                    Color::linear_rgb(1.0, 0.0, 0.0),
                )
                .resolution(16);

            continue;
        }

        if !settings.dev_settings.show_ray_casters {
            continue;
        }

        gizmos.ray(
            transform.translation(),
            transform.rotation() * caster.direction,
            Color::linear_rgb(0.0, 0.0, 1.0),
        );

        caster.result = None
    }
}
