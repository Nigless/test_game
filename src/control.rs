use bevy::{
    app::Plugin,
    ecs::{component::Component, system::Resource},
    input::{keyboard::KeyCode, mouse::MouseMotion},
    math::Vec2,
    prelude::*,
};

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Control;

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct Bindings {
    pub move_forward: KeyCode,
    pub move_left: KeyCode,
    pub move_right: KeyCode,
    pub move_backward: KeyCode,
    pub jump: KeyCode,
    pub crouch: KeyCode,
    pub run: KeyCode,
    pub pause: KeyCode,
}

impl Default for Bindings {
    fn default() -> Self {
        Self {
            move_forward: KeyCode::KeyW,
            move_left: KeyCode::KeyA,
            move_right: KeyCode::KeyD,
            move_backward: KeyCode::KeyS,
            jump: KeyCode::Space,
            crouch: KeyCode::ControlLeft,
            run: KeyCode::ShiftLeft,
            pause: KeyCode::Escape,
        }
    }
}

#[derive(Resource, Default, Reflect, PartialEq)]
#[reflect(Resource)]
pub struct Input {
    pub moving: Vec2,
    pub looking_around: Vec2,
    pub jumping: bool,
    pub running: bool,
    pub crouching: bool,
    pub pausing: bool,
}

pub struct ControlPlugin;

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Control>()
            .register_type::<Input>()
            .insert_resource(Input::default())
            .register_type::<Bindings>()
            .insert_resource(Bindings::default())
            .add_systems(First, update);
    }
}

fn update(
    mut mouse: EventReader<MouseMotion>,
    keyboard: Res<ButtonInput<KeyCode>>,
    controls: Res<Bindings>,
    mut input: ResMut<Input>,
) {
    let mut result = Input::default();

    for event in mouse.read().into_iter() {
        result.looking_around += Vec2::new(-event.delta.x, -event.delta.y);
    }

    if keyboard.pressed(controls.move_left) {
        result.moving += Vec2::new(-1.0, 0.0);
    }

    if keyboard.pressed(controls.move_right) {
        result.moving += Vec2::new(1.0, 0.0);
    }

    if keyboard.pressed(controls.move_forward) {
        result.moving += Vec2::new(0.0, -1.0);
    }

    if keyboard.pressed(controls.move_backward) {
        result.moving += Vec2::new(0.0, 1.0);
    }

    result.jumping = keyboard.just_pressed(controls.jump);
    result.running = keyboard.pressed(controls.run);
    result.crouching = keyboard.pressed(controls.crouch);

    result.pausing = keyboard.just_pressed(controls.pause);

    if *input != result {
        *input = result;
    }
}
