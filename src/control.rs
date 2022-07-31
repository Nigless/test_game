use crate::{entities::player::moving::Moving, physics::Physics};
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;

#[derive(Component)]
pub struct Control;

pub struct ControlPlugin;

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(startup)
            .add_system(movement)
            .add_system(rotation);
    }
}

fn startup(mut windows: ResMut<Windows>) {
    if let Some(window) = windows.get_primary_mut() {
        window.set_cursor_visibility(false);
    }
}

fn rotation(
    mut entity: Query<&mut Transform, (With<Control>, With<Physics>)>,
    mut cursor: EventReader<MouseMotion>,
    mut windows: ResMut<Windows>,
) {
    if let None = windows.get_primary_mut() {
        return;
    }
    let window = windows.get_primary_mut().unwrap();
    if !window.is_focused() {
        return;
    }

    if let Err(_) = entity.get_single() {
        return;
    }
    let mut entity = entity.single_mut();

    for event in cursor.iter() {
        let mut rotation = Quat::from_rotation_y(-(*event.delta).x * 0.002) * entity.rotation;
        let rotate_x = rotation * Quat::from_rotation_x(-(*event.delta).y * 0.002);
        if (rotate_x * Vec3::Y).y > 0.0 {
            rotation = rotate_x
        }
        entity.rotation = rotation;
    }
    window.set_cursor_position(Vec2::new(window.width() / 2., window.height() / 2.));
}

fn movement(
    mut entity: Query<(&mut Transform, &mut Physics, &mut Moving), With<Control>>,
    keyboard: Res<Input<KeyCode>>,
) {
    if let Err(_) = entity.get_single() {
        return;
    }
    let (transform, mut physics, mut moving) = entity.single_mut();

    let mut mov = Vec3::ZERO;

    if keyboard.just_pressed(KeyCode::Space) {
        if transform.translation.y <= 0.0 {
            physics.impulse.y += 40.0
        }
    }

    if keyboard.just_pressed(KeyCode::LControl) {
        moving.ducking = true;
        if transform.translation.y > 0.0 && physics.impulse.y < 10.0 && !moving.pushing_down {
            moving.pushing_down = true;
            physics.impulse.y -= 40.0;
        }
    }

    if keyboard.just_released(KeyCode::LControl) {
        if moving.sliding {
            moving.sliding = false
        }
        moving.ducking = false
    }

    if keyboard.pressed(KeyCode::D) {
        if !moving.sliding {
            mov += Vec3::new(transform.right().x, 0.0, transform.right().z).normalize();
            moving.moving = true;
        }
    }
    if keyboard.pressed(KeyCode::A) {
        if !moving.sliding {
            mov += Vec3::new(transform.left().x, 0.0, transform.left().z).normalize();
            moving.moving = true;
        }
    }

    if keyboard.pressed(KeyCode::S) {
        if !moving.sliding {
            mov += Vec3::new(transform.back().x, 0.0, transform.back().z).normalize();
            moving.moving = true;
        }
    }

    if keyboard.pressed(KeyCode::W) {
        if moving.sliding {
            mov += Vec3::new(physics.impulse.x, 0.0, physics.impulse.z).normalize();
        } else if moving.ducking && transform.translation.y <= 0.0 {
            moving.sliding = true;
            moving.ducking = false;
            mov += Vec3::new(transform.forward().x, 0.0, transform.forward().z).normalize();
        } else {
            mov += Vec3::new(transform.forward().x, 0.0, transform.forward().z).normalize();
            moving.moving = true;
        }
    }

    if mov == Vec3::ZERO {
        moving.moving = false;
        if moving.sliding {
            moving.sliding = false;
            moving.ducking = true;
        }
        if transform.translation.y <= 0.0 {
            physics.mov_x(0.0);
            physics.mov_z(0.0);
        }
    } else {
        let mut mov = mov.normalize();
        if moving.sliding {
            mov *= 30.0
        } else if moving.ducking {
            mov *= 5.0
        } else {
            mov *= 15.0
        }

        physics.mov_x(mov.x);
        physics.mov_z(mov.z);
    }

    if transform.translation.y <= 0.0 {
        moving.pushing_down = false;
    }
}
