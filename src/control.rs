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
    pub swim_up: KeyCode,
    pub swim_down: KeyCode,
    pub jump: KeyCode,
    pub crouch: KeyCode,
    pub run: KeyCode,
    pub pause: KeyCode,
    pub switch_full_screen: KeyCode,
    pub mouse_sensitivity: f32,
    pub grub: KeyCode,
    pub pickup: KeyCode,
    pub drop: KeyCode,
    pub save: KeyCode,
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
            switch_full_screen: KeyCode::F11,
            mouse_sensitivity: 0.001,
            swim_up: KeyCode::Space,
            swim_down: KeyCode::ControlLeft,
            grub: KeyCode::KeyF,
            pickup: KeyCode::KeyE,
            drop: KeyCode::KeyQ,
            save: KeyCode::F5,
        }
    }
}

#[derive(Resource, Default, Reflect, PartialEq)]
#[reflect(Resource)]
pub struct Input {
    pub moving: Vec2,
    looking: Vec2,
    jumping: bool,
    pub running: bool,
    pub swimming_up: bool,
    pub swimming_down: bool,
    pub crouching: bool,
    pub pausing: bool,
    saving: bool,
    full_screen_switching: bool,
}

impl Input {
    pub fn jumping(&mut self) -> bool {
        if self.jumping {
            self.jumping = false;
            return true;
        }

        false
    }

    pub fn saving(&mut self) -> bool {
        if self.saving {
            self.saving = false;
            return true;
        }

        false
    }

    pub fn full_screen_switching(&mut self) -> bool {
        if self.full_screen_switching {
            self.full_screen_switching = false;
            return true;
        }

        false
    }

    pub fn looking(&mut self) -> Vec2 {
        let result = self.looking;

        self.looking = Vec2::ZERO;

        result
    }
}

#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub struct ControlSystems;

pub struct ControlPlugin;

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Control>();

        app.register_type::<Bindings>()
            .insert_resource(Bindings::default());

        app.register_type::<Input>()
            .insert_resource(Input::default());

        app.add_systems(PreUpdate, update.in_set(ControlSystems));
    }
}

fn update(
    mut mouse: EventReader<MouseMotion>,
    keyboard: Res<ButtonInput<KeyCode>>,
    controls: Res<Bindings>,
    mut input: ResMut<Input>,
) {
    for event in mouse.read().into_iter() {
        input.looking += Vec2::new(-event.delta.x, -event.delta.y) * controls.mouse_sensitivity;
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

    input.jumping = input.jumping || keyboard.just_pressed(controls.jump);
    input.full_screen_switching =
        input.full_screen_switching || keyboard.just_pressed(controls.switch_full_screen);
    input.running = keyboard.pressed(controls.run);
    input.crouching = keyboard.pressed(controls.crouch);
    input.swimming_up = keyboard.pressed(controls.swim_up);
    input.swimming_down = keyboard.pressed(controls.swim_down);
    // input.saving = keyboard.just_pressed(controls.save) || input.saving;

    input.pausing = keyboard.just_pressed(controls.pause);
}
