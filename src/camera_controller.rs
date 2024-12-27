use bevy::prelude::*;

#[derive(Component)]
pub struct Spectate;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct CameraController {
    pub target: Entity,
}

impl CameraController {
    pub fn new(target: Entity) -> Self {
        Self { target }
    }
}

#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub enum CameraControllerSystems {
    Resolve,
}

pub struct CameraControllerPlugin;

impl Plugin for CameraControllerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CameraController>().add_systems(
            First,
            (clean_up, resolve).in_set(CameraControllerSystems::Resolve),
        );
    }
}

type CameraQuery<'world, 'state, 'q> = Query<'world, 'state, &'q mut Camera, With<Camera3d>>;

fn clean_up(removed: RemovedComponents<Spectate>, mut camera_q: CameraQuery) {
    if removed.is_empty() {
        return;
    }

    for mut camera in camera_q.iter_mut() {
        camera.is_active = false
    }
}

fn resolve(
    entity_q: Query<
        &CameraController,
        (
            Or<(Added<Spectate>, Added<CameraController>)>,
            With<Spectate>,
        ),
    >,
    mut camera_q: CameraQuery,
) {
    if entity_q.is_empty() {
        return;
    }

    for mut camera in camera_q.iter_mut() {
        camera.is_active = false
    }

    let controller = entity_q
        .get_single()
        .expect("only one entity can have a Spectate component");

    let mut camera = camera_q
        .get_mut(controller.target)
        .expect("CameraController target doesn't exist or doesn't have a Camera component");

    camera.is_active = true;
}
