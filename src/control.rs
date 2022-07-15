use crate::components::physics::Physics;
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
    fn startup(mut windows: ResMut<Windows>) {
        if let Some(window) = windows.get_primary_mut() {
            window.set_cursor_visibility(false);

            if !window.is_focused() {
                return;
            }

            window.set_cursor_position(Vec2::new(window.width() / 2., window.height() / 2.));
        }
    }

    fn rotation(
        mut entity: Query<&mut Transform, (With<Control>, With<Physics>)>,
        mut cursor: EventReader<MouseMotion>,
        mut windows: ResMut<Windows>,
    ) {
        if let Some(window) = windows.get_primary_mut() {
            if !window.is_focused() {
                return;
            }

            window.set_cursor_position(Vec2::new(window.width() / 2., window.height() / 2.));
        }

        if let Err(_) = entity.get_single() {
            return;
        }
        let mut entity = entity.single_mut();

        for event in cursor.iter() {
            entity.rotation = Quat::from_rotation_y(-(*event.delta).x * 0.002)
                * entity.rotation
                * Quat::from_rotation_x(-(*event.delta).y * 0.002)
        }
    }

    fn movement(
        mut entity: Query<(&mut Transform, &mut Physics), With<Control>>,
        keyboard: Res<Input<KeyCode>>,
    ) {
        if let Err(_) = entity.get_single() {
            return;
        }
        let (mut entity, mut physics) = entity.single_mut();

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
            physics.mov(mov.normalize());
        } else {
            physics.impulse *= 0.9;
        }
    }
}
