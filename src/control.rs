use crate::head::{Head, WithHead};
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy_rapier3d::prelude::Velocity;

#[derive(Component)]
pub struct Control;

pub struct ControlPlugin;

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut App) {
        app
        // .add_startup_system(startup)
            .add_system(movement)
            .add_system(rotation)
            .add_system(head_rotation);
    }
}

fn startup(mut windows: ResMut<Windows>) {
    if let Some(window) = windows.get_primary_mut() {
        window.set_cursor_visibility(false);
    }
}

fn rotation(
    mut entity: Query<&mut Transform, (With<Control>, Without<WithHead>, Without<Head>)>,
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
        let mut rotation = Quat::from_rotation_y(-(event.delta).x * 0.002) * entity.rotation;
        let rotate_x = rotation * Quat::from_rotation_x(-(event.delta).y * 0.002);
        if (rotate_x * Vec3::Y).y > 0.0 {
            rotation = rotate_x
        }
        entity.rotation = rotation;
    }
    // window.set_cursor_position(Vec2::new(window.width() / 2., window.height() / 2.));
}

fn head_rotation(
    mut entity_q: Query<(&mut Transform, &Head), With<Control>>,
    mut transform_q: Query<&mut Transform, (Without<Head>, Without<Control>)>,
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

    if let Err(_) = entity_q.get_single() {
        return;
    }
    let (mut entity_tfm, entity_head) = entity_q.get_single_mut().unwrap();

    for event in cursor.iter() {
        let rotation = Quat::from_rotation_y(-(event.delta).x * 0.002) * entity_tfm.rotation;
        entity_tfm.rotation = rotation;

        let mut head_tfm = transform_q.get_mut(entity_head.target).unwrap();

        let rotation = head_tfm.rotation * Quat::from_rotation_x(-(event.delta).y * 0.002);
        if (rotation * Vec3::Y).y > 0.0 {
            head_tfm.rotation = rotation;
        }
    }

    // window.set_cursor_position(Vec2::new(window.width() / 2., window.height() / 2.));
}

fn movement(
    mut entity: Query<(&mut Transform, &mut Velocity), With<Control>>,
    keyboard: Res<Input<KeyCode>>,
) {
    if let Err(_) = entity.get_single() {
        return;
    }
    let (transform, mut velocity) = entity.single_mut();

    let mut mov = Vec3::ZERO;

    if keyboard.pressed(KeyCode::D) {
        mov += Vec3::new(transform.right().x, 0.0, transform.right().z).normalize();
    }
    if keyboard.pressed(KeyCode::A) {
        mov += Vec3::new(transform.left().x, 0.0, transform.left().z).normalize();
    }

    if keyboard.pressed(KeyCode::S) {
        mov += Vec3::new(transform.back().x, 0.0, transform.back().z).normalize();
    }

    if keyboard.pressed(KeyCode::W) {
        mov += Vec3::new(transform.forward().x, 0.0, transform.forward().z).normalize();
    }

    velocity.linvel = mov.normalize_or_zero() * 20.0;
}
