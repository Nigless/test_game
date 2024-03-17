use bevy::app::MainScheduleOrder;
use bevy::ecs::schedule::ScheduleLabel;
use bevy::render::camera::RenderTarget;
use bevy::{ecs::reflect, input::mouse::MouseMotion};
use bevy::{prelude::*, render::camera};

#[derive(Component)]
pub struct Spectate;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct CameraController {
    pub target: Entity,
}

pub struct CameraControllerPlugin;

impl Plugin for CameraControllerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CameraController>()
            .add_systems(PostStartup, resolve)
            .add_systems(PreUpdate, (clean_up, update.after(clean_up)));
    }
}

fn resolve(entity_q: Query<&CameraController, With<Spectate>>, mut camera_q: Query<&mut Camera>) {
    for mut camera in camera_q.iter_mut() {
        camera.is_active = false
    }

    if entity_q.is_empty() {
        return;
    }

    let controller = entity_q
        .get_single()
        .expect("only one entity can have a Spectate component");

    let mut camera = camera_q
        .get_mut(controller.target)
        .expect("CameraController target doesn't exist or doesn't have a Camera component");

    camera.is_active = true;
}

fn update(entity_q: Query<&CameraController, Added<Spectate>>, mut camera_q: Query<&mut Camera>) {
    if entity_q.is_empty() {
        return;
    }

    let controller = entity_q
        .get_single()
        .expect("only one entity can have a Spectate component");

    let mut camera = camera_q
        .get_mut(controller.target)
        .expect("CameraController target doesn't exist or doesn't have a Camera component");

    camera.is_active = true;
}

fn clean_up(entity_q: RemovedComponents<Spectate>, mut camera_q: Query<&mut Camera>) {
    if entity_q.is_empty() {
        return;
    }

    for mut camera in camera_q.iter_mut() {
        camera.is_active = false
    }
}
