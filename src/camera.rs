use crate::control::Control;
use bevy::{prelude::*, render::camera::Projection};
use std::f32::consts::PI;

#[derive(Component)]
struct Camera();

#[derive(Component)]
struct CameraTarget();

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(startup).add_system(follow);
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
        .insert(Camera());
}

fn follow(
    mut camera_q: Query<&mut Transform, With<Camera>>,
    mut target_q: Query<&Transform, (With<CameraTarget>, Without<Camera>)>,
) {
    if let Err(_) = target_q.get_single() {
        return;
    }
    let target_t = target_q.single_mut();

    let mut camera_t = camera_q.single_mut();

    camera_t.translation = target_t.translation;
    camera_t.rotation = target_t.rotation;
}
