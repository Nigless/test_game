use bevy::prelude::{
    default, App, Commands, Component, Entity, PerspectiveCameraBundle, PerspectiveProjection,
    Plugin, Query, Transform, With, Without,
};
use std::f32::consts::PI;

use crate::{
    components::{name::Name, physics::Physics},
    control::Control,
};

#[derive(Component)]
struct Cam {
    target: Option<Entity>,
}

pub struct CameraPlugin();

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(Self::startup)
            .add_system(Self::target)
            .add_system(Self::follow);
    }
}

impl CameraPlugin {
    fn startup(mut commands: Commands) {
        commands
            .spawn_bundle(PerspectiveCameraBundle {
                perspective_projection: PerspectiveProjection {
                    fov: PI * 0.4,
                    ..default()
                },
                ..default()
            })
            .insert(Cam { target: None });
    }

    fn target(entity_q: Query<Entity, With<Control>>, mut camera_q: Query<&mut Cam>) {
        let mut camera = camera_q.single_mut();
        if let Some(_) = camera.target {
            return;
        }

        if let Ok(entity) = entity_q.get_single() {
            camera.target = Some(entity)
        }
    }

    fn follow(
        mut camera_q: Query<(&Cam, &mut Transform)>,
        target_q: Query<&Transform, Without<Cam>>,
    ) {
        let (camera, mut transform) = camera_q.single_mut();

        if let Some(target) = camera.target {
            if let Ok(component) = target_q.get_component::<Transform>(target) {
                transform.translation = component.translation;
                transform.translation.y += 5.0;
                transform.rotation = component.rotation
            };
        };
    }
}
