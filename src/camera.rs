use crate::{control::Control, head::Head};
use bevy::{prelude::*, render::camera::Projection};
use std::f32::consts::PI;

#[derive(Component)]
struct Camera;

#[derive(Component)]
pub struct CameraTarget;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(startup)
            .add_system(follow)
            .add_system(follow_head);
    }
}

fn startup(mut commands: Commands) {
    commands
        .spawn_bundle(Camera3dBundle {
            projection: Projection::Perspective(PerspectiveProjection {
                fov: PI * 0.4,
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(Camera);
}

fn follow(
    mut camera_q: Query<&mut Transform, With<Camera>>,
    mut target_q: Query<&Transform, (With<CameraTarget>, Without<Camera>, Without<Head>)>,
) {
    if let Err(_) = target_q.get_single() {
        return;
    }
    let target_t = target_q.single_mut();

    let mut camera_t = camera_q.single_mut();

    camera_t.translation = target_t.translation;
    camera_t.rotation = target_t.rotation;
}

fn follow_head(
    mut camera_q: Query<&mut Transform, With<Camera>>,
    mut target_q: Query<&Head, (With<CameraTarget>, Without<Camera>)>,
    mut transform_q: Query<&GlobalTransform, (Without<Head>, Without<Camera>)>,
) {
    if let Err(_) = target_q.get_single() {
        return;
    }
    let entity_head = target_q.single_mut();

    let head_tfm = transform_q
        .get_mut(entity_head.target)
        .unwrap()
        .compute_transform();

    let mut camera_tfm = camera_q.single_mut();

    camera_tfm.translation = head_tfm.translation;
    camera_tfm.rotation = head_tfm.rotation;
}
