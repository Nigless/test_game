use crate::head::Head;
use bevy::{prelude::*, render::camera::Projection, transform::TransformSystem};
use std::f32::consts::PI;

#[derive(Component)]
pub struct Camera;

#[derive(Component)]
pub struct CameraTarget;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(startup).add_system_to_stage(
            CoreStage::PostUpdate,
            follow.after(TransformSystem::TransformPropagate),
        );
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
    target_q: Query<(&Transform, Option<&Head>), (With<CameraTarget>, Without<Camera>)>,
    transform_q: Query<&GlobalTransform, (Without<Head>, Without<Camera>)>,
) {
    let mut camera_tfm = camera_q.single_mut();
    let (target_tfm, target_head) = match target_q.get_single() {
        Ok(e) => e,
        Err(_) => return,
    };

    let target_tfm = match target_head {
        Some(e) => transform_q.get(e.target).unwrap().compute_transform(),
        None => *target_tfm,
    };

    camera_tfm.translation = target_tfm.translation;
    camera_tfm.rotation = target_tfm.rotation;
}
