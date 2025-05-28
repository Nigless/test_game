use bevy::{
    app::{App, Plugin, Startup},
    color::Color,
    core::Name,
    core_pipeline::core_2d::{Camera2d, Camera2dBundle},
    ecs::{schedule::IntoSystemConfigs, system::Commands, world::OnInsert},
    render::camera::Camera,
    state::{
        app::AppExtStates,
        commands,
        condition::in_state,
        state::{OnEnter, OnExit, States},
    },
    ui::{widget::Button, *},
    utils::default,
};

pub struct TitleStatePlugin<T: States>(pub T);

impl<T: States> Plugin for TitleStatePlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(self.0.clone()), enter)
            .add_systems(OnExit(self.0.clone()), exit);
    }
}

fn enter(mut commands: Commands) {
    commands.spawn(Camera2d::default());

    commands.spawn((
        Button,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            border: UiRect::all(Val::Px(5.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BorderColor(Color::BLACK),
        BorderRadius::MAX,
        BackgroundColor(Color::BLACK),
    ));
}

fn exit(mut commands: Commands) {
    commands.spawn(Camera2d::default());

    commands.spawn((
        Button,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            border: UiRect::all(Val::Px(5.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BorderColor(Color::BLACK),
        BorderRadius::MAX,
        BackgroundColor(Color::BLACK),
    ));
}
