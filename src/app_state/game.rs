use bevy::{
    app::{App, Plugin, Startup},
    core::Name,
    ecs::{
        schedule::IntoSystemConfigs,
        system::{Commands, Res},
        world::OnInsert,
    },
    hierarchy::BuildChildren,
    math::Vec3,
    state::{
        app::AppExtStates,
        commands,
        condition::in_state,
        state::{OnEnter, States},
    },
    transform::components::Transform,
};
use bevy_inspector_egui::egui::util::id_type_map::TypeId;
use bevy_rapier3d::prelude::Velocity;

use crate::{
    control::Control,
    entities::{block::BlockBundle, fireball::Fireball, player::Player, traffic_cone::TrafficCone},
    library::Spawnable,
    prefab::Prefab,
    saves::Serializable,
    scenes::Scenes,
};

pub struct GameStatePlugin<T: States>(pub T);

impl<T: States> Plugin for GameStatePlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(self.0.clone()), enter);
    }
}

fn enter(mut commands: Commands, scenes: Res<Scenes>) {
    let commands = &mut commands;

    // commands
    //     .spawn((
    //         Serializable::default().with::<Prefab>(),
    //         Prefab::new("test_scene"),
    //     ))
    //     .set_parent(scenes.current);

    Player::default()
        .spawn(commands)
        .insert((Transform::from_xyz(0.0, 3.0, 0.0), Control))
        .set_parent(scenes.current);

    Fireball
        .spawn(commands)
        .insert((
            Transform::from_xyz(0.0, 1.0, 3.0),
            Velocity::linear(Vec3::X),
        ))
        .set_parent(scenes.current);

    Fireball
        .spawn(commands)
        .insert(Transform::from_xyz(0.0, 1.0, -3.0))
        .set_parent(scenes.current);

    BlockBundle::default()
        .spawn(commands)
        .insert(Transform::from_xyz(-4.0, 3.0, 24.0))
        .set_parent(scenes.current);

    BlockBundle::default()
        .spawn(commands)
        .insert(Transform::from_xyz(4.0, 3.0, 16.0))
        .set_parent(scenes.current);

    BlockBundle::new(1.0, 0.5, 4.0)
        .with_mass(100.0)
        .spawn(commands)
        .insert(Transform::from_xyz(4.0, 3.0, 24.0))
        .set_parent(scenes.current);

    BlockBundle::new(0.5, 0.5, 0.5)
        .with_mass(25.0 / 2.0)
        .spawn(commands)
        .insert(Transform::from_xyz(0.0, 2.0, 20.0))
        .set_parent(scenes.current);

    BlockBundle::new(2.0, 0.1, 2.0)
        .with_mass(100.0)
        .spawn(commands)
        .insert(Transform::from_xyz(-20.0, 1.0, 2.0))
        .set_parent(scenes.current);

    BlockBundle::new(0.2, 0.2, 0.2)
        .with_mass(1.0)
        .spawn(commands)
        .insert(Transform::from_xyz(-22.0, 1.0, 2.0))
        .set_parent(scenes.current);

    BlockBundle::new(1.0, 1.0, 1.0)
        .with_mass(800.0)
        .spawn(commands)
        .insert(Transform::from_xyz(-22.0, 1.0, 4.0))
        .set_parent(scenes.current);

    // TrafficCone
    //     .spawn(commands)
    //     .insert(Transform::from_xyz(0.0, 1.0, 0.0))
    //     .set_parent(scenes.current);
}
