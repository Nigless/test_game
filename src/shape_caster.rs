use bevy::{
    ecs::{
        component::{ComponentHooks, ComponentId, StorageType},
        world::DeferredWorld,
    },
    prelude::*,
};
use bevy_rapier3d::{
    plugin::RapierContext,
    prelude::{Collider, Group, QueryFilter, ShapeCastOptions, SolverGroups},
};

#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub struct ShapeCasterSystems;

#[derive(Reflect)]
pub struct CasterResult {
    pub entity: Entity,
    pub distance: f32,
    pub normal: Vec3,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct ShapeCasterFixed;

#[derive(Reflect, Component)]
#[component(on_add = insert_tag_for_fixed)]
#[reflect(Component)]
#[require(Transform)]
pub struct ShapeCaster {
    #[reflect(ignore)]
    pub collider: Collider,
    pub direction: Vec3,
    pub result: Option<CasterResult>,
    pub fixed_update: bool,
    exclude: Option<Entity>,
}

impl ShapeCaster {
    pub fn new(collider: Collider, direction: Vec3) -> Self {
        Self {
            direction,
            result: None,
            fixed_update: false,
            exclude: None,
            collider,
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

fn insert_tag_for_fixed(mut world: DeferredWorld<'_>, entity: Entity, _: ComponentId) {
    let fixed_update = world.get::<ShapeCaster>(entity).unwrap().fixed_update;

    if fixed_update {
        world.commands().entity(entity).insert(ShapeCasterFixed);
    }
}

pub struct ShapeCasterPlugin;

impl Plugin for ShapeCasterPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ShapeCaster>()
            .register_type::<ShapeCasterFixed>()
            .add_systems(
                FixedPreUpdate,
                update::<ShapeCasterFixed>.in_set(ShapeCasterSystems),
            )
            .add_systems(PreUpdate, update::<ShapeCaster>.in_set(ShapeCasterSystems));
    }
}

fn update<T: Component>(
    rapier: Single<&RapierContext>,
    mut entity_q: Query<(&mut ShapeCaster, &GlobalTransform), (Without<RapierContext>, With<T>)>,
) {
    let mut filter = QueryFilter::default().exclude_sensors();

    for (mut caster, transform) in entity_q.iter_mut() {
        filter.exclude_collider = caster.exclude;

        if let Some((entity, time_of_impact, normal)) = rapier
            .cast_shape(
                transform.translation(),
                transform.rotation(),
                caster.direction,
                &caster.collider,
                ShapeCastOptions {
                    max_time_of_impact: 1.0,
                    ..default()
                },
                filter,
            )
            .map_or(None, |(entity, hit)| {
                hit.details
                    .map(|details| (entity, hit.time_of_impact, details.normal1))
            })
        {
            caster.result = Some(CasterResult {
                entity: rapier.collider_parent(entity).unwrap_or(entity),
                distance: caster.direction.length() * time_of_impact,
                normal,
            });

            continue;
        }

        caster.result = None
    }
}
