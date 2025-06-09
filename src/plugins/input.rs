use bevy::{
    app::Plugin,
    ecs::{component::Component, system::Resource},
    input::{keyboard::KeyCode, mouse::MouseMotion},
    log::tracing_subscriber::reload::Handle,
    math::Vec2,
    prelude::*,
    state::commands,
    utils::HashMap,
};

use crate::plugins::settings::Settings;

#[derive(Event)]
pub struct JumpingPressedEvent;

#[derive(Event)]
pub struct FullScreenSwitchingPressedEvent;

#[derive(Event)]
pub struct PausingPressedEvent;

#[derive(Event)]
pub struct SavingPressedEvent;

#[derive(Event)]
pub struct LoadingPressedEvent;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Control;

#[derive(Resource, Reflect)]
#[reflect(Resource)]
struct Bindings {
    items: HashMap<String, KeyCode>,
}

impl Bindings {
    fn with(mut self, name: &str, key: KeyCode) -> Self {
        self.items.insert(name.to_owned(), key);
        self
    }

    fn get(&self, name: &str) -> KeyCode {
        self.items.get(name).cloned().unwrap()
    }
}

impl Default for Bindings {
    fn default() -> Self {
        Self { items: default() }
            .with("move_forward", KeyCode::KeyW)
            .with("move_left", KeyCode::KeyA)
            .with("move_right", KeyCode::KeyD)
            .with("move_backward", KeyCode::KeyS)
            .with("swim_up", KeyCode::Space)
            .with("swim_down", KeyCode::ControlLeft)
            .with("jump", KeyCode::Space)
            .with("crouch", KeyCode::ControlLeft)
            .with("run", KeyCode::ShiftLeft)
            .with("pause", KeyCode::Escape)
            .with("switch_full_screen", KeyCode::F11)
            .with("grub", KeyCode::KeyF)
            .with("pickup", KeyCode::KeyE)
            .with("drop", KeyCode::KeyQ)
            .with("save", KeyCode::F5)
            .with("load", KeyCode::F6)
    }
}

#[derive(Resource, Default, Reflect, PartialEq)]
#[reflect(Resource)]
pub struct Input {
    pub moving: Vec2,
    pub looking: Vec2,
    pub running: bool,
    pub swimming_up: bool,
    pub swimming_down: bool,
    pub crouching: bool,
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Control>();

        app.register_type::<Bindings>()
            .insert_resource(Bindings::default());

        app.register_type::<Input>()
            .insert_resource(Input::default());

        app.add_systems(First, update);
    }
}

fn update(
    mut mouse: EventReader<MouseMotion>,
    keyboard: Res<ButtonInput<KeyCode>>,
    bindings: Res<Bindings>,
    config: Res<Settings>,
    mut input: ResMut<Input>,
    mut commands: Commands,
) {
    input.looking = Vec2::ZERO;

    for event in mouse.read().into_iter() {
        input.looking += Vec2::new(-event.delta.x, -event.delta.y) * config.mouse_sensitivity;
    }

    input.moving = Vec2::ZERO;

    if keyboard.pressed(bindings.get("move_left")) {
        input.moving += Vec2::new(-1.0, 0.0);
    }

    if keyboard.pressed(bindings.get("move_right")) {
        input.moving += Vec2::new(1.0, 0.0);
    }

    if keyboard.pressed(bindings.get("move_forward")) {
        input.moving += Vec2::new(0.0, -1.0);
    }

    if keyboard.pressed(bindings.get("move_backward")) {
        input.moving += Vec2::new(0.0, 1.0);
    }

    input.running = keyboard.pressed(bindings.get("run"));
    input.crouching = keyboard.pressed(bindings.get("crouch"));
    input.swimming_up = keyboard.pressed(bindings.get("swim_up"));
    input.swimming_down = keyboard.pressed(bindings.get("swim_down"));

    if keyboard.just_pressed(bindings.get("jump")) {
        commands.trigger(JumpingPressedEvent);
    }

    if keyboard.just_pressed(bindings.get("switch_full_screen")) {
        commands.trigger(FullScreenSwitchingPressedEvent);
    }

    if keyboard.just_pressed(bindings.get("save")) {
        commands.trigger(SavingPressedEvent);
    }

    if keyboard.just_pressed(bindings.get("load")) {
        commands.trigger(LoadingPressedEvent);
    }

    if keyboard.just_pressed(bindings.get("pause")) {
        commands.trigger(PausingPressedEvent);
    }
}
