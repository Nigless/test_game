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

#[derive(Resource, Default, Reflect)]
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
            .add_systems(PreUpdate, update);
    }
}

fn update(
    mut mouse: EventReader<MouseMotion>,
    keyboard: Res<ButtonInput<KeyCode>>,
    controls: Res<Bindings>,
    mut input: ResMut<Input>,
) {
    input.looking_around = Vec2::ZERO;

    for event in mouse.read().into_iter() {
        input.looking_around += Vec2::new(-event.delta.x, -event.delta.y);
    }

    input.moving = Vec2::ZERO;

    if keyboard.pressed(controls.move_left) {
        input.moving += Vec2::new(-1.0, 0.0);
    }

    if keyboard.pressed(controls.move_right) {
        input.moving += Vec2::new(1.0, 0.0);
    }

    if keyboard.pressed(controls.move_forward) {
        input.moving += Vec2::new(0.0, -1.0);
    }

    if keyboard.pressed(controls.move_backward) {
        input.moving += Vec2::new(0.0, 1.0);
    }

    input.jumping = keyboard.just_pressed(controls.jump);
    input.running = keyboard.pressed(controls.run);
    input.crouching = keyboard.pressed(controls.crouch);

    input.pausing = keyboard.just_pressed(controls.pause);
}
