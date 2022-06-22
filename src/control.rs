use crate::components::speed::Speed;
use bevy::ecs::bundle::Bundle;
use bevy::input::mouse::MouseMotion;
use bevy::render::camera::Camera3d;
use bevy::{ecs::query, prelude::*};
use core::ops::Mul;
use std::f32::consts::PI;

#[derive(Component)]
pub struct Control();

pub struct ControlPlugin();

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(Self::startup)
            .add_system(Self::movement)
            .add_system(Self::rotation);
    }
}

impl ControlPlugin {
    fn startup(mut commands: Commands) {
        commands
            .spawn_bundle(PerspectiveCameraBundle {
                perspective_projection: PerspectiveProjection {
                    fov: PI * 0.4,
                    ..default()
                },
                ..default()
            })
            .insert(Control());
    }

    fn rotation(
        mut camera: Query<&mut Transform, (With<Control>, With<Camera3d>)>,
        mut entity: Query<&mut Transform, (With<Control>, With<Speed>, Without<Camera3d>)>,
        mut cursor: EventReader<MouseMotion>,
    ) {
        if let Err(_) = entity.get_single() {
            return;
        }
        let mut entity = entity.single_mut();
        let mut camera = camera.single_mut();

        for event in cursor.iter() {
            entity.rotation = Quat::from_rotation_y(-(*event.delta).x * 0.002)
                * entity.rotation
                * Quat::from_rotation_x(-(*event.delta).y * 0.002)
        }

        camera.rotation = entity.rotation
    }

    fn movement(
        mut camera: Query<&mut Transform, (With<Control>, With<Camera3d>)>,
        mut entity: Query<(&mut Transform, &Speed), (With<Control>, Without<Camera3d>)>,
        keyboard: Res<Input<KeyCode>>,
        time: Res<Time>,
    ) {
        if let Err(_) = entity.get_single() {
            return;
        }
        let (mut entity, speed) = entity.single_mut();
        let mut camera = camera.single_mut();

        let mut mov = Vec3::ZERO;
        if keyboard.pressed(KeyCode::D) {
            mov += Vec3::new(entity.right().x, 0.0, entity.right().z).normalize();
        }
        if keyboard.pressed(KeyCode::A) {
            mov += Vec3::new(entity.left().x, 0.0, entity.left().z).normalize();
        }

        if keyboard.pressed(KeyCode::Space) {
            mov += Vec3::new(0.0, 1.0, 0.0);
        }
        if keyboard.pressed(KeyCode::LShift) {
            mov += Vec3::new(0.0, -1.0, 0.0);
        }

        if keyboard.pressed(KeyCode::S) {
            mov += Vec3::new(entity.back().x, 0.0, entity.back().z).normalize();
        }
        if keyboard.pressed(KeyCode::W) {
            mov += Vec3::new(entity.forward().x, 0.0, entity.forward().z).normalize();
        }

        if mov != Vec3::ZERO {
            entity.translation += mov.normalize() * f32::from(speed.value) * time.delta_seconds();
        }

        camera.translation = entity.translation;
    }
}
