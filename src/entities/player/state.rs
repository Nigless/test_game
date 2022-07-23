use bevy::ecs::component::Component;

pub enum Action {
    DuckingOn,
    DuckingOff,
    JumpingOn,
    JumpingOff,
    MovingOn,
    MovingOff,
}

pub trait IState {
    fn dispatch(&mut self, action: Action);
}

#[derive(Component, Default)]
pub struct State {
    ducking: bool,
    jumping: bool,
    moving: bool,
    push_down: bool,
}

impl IState for State {
    fn dispatch(&mut self, action: Action) {
        match action {
            Action::DuckingOn => {
                if self.is_jumping() {
                    self.set_push_down(true);
                    return;
                }
                self.set_ducking(true)
            }
            Action::DuckingOff => self.set_ducking(false),
            Action::JumpingOn => self.set_jumping(true),
            Action::JumpingOff => {
                self.set_jumping(false);
                self.set_push_down(false);
            }
            Action::MovingOn => self.set_moving(true),
            Action::MovingOff => self.set_moving(false),
        }
    }
}

impl State {
    pub fn set_ducking(&mut self, value: bool) {
        if self.ducking == value {
            return;
        }
        self.ducking = value;
    }

    pub fn set_jumping(&mut self, value: bool) {
        if self.jumping == value {
            return;
        }
        self.jumping = value;
    }

    pub fn set_push_down(&mut self, value: bool) {
        if self.push_down == value {
            return;
        }
        self.push_down = value;
    }

    pub fn set_moving(&mut self, value: bool) {
        if self.moving == value {
            return;
        }
        self.moving = value;
    }

    pub fn is_ducking(&self) -> bool {
        self.ducking
    }

    pub fn is_jumping(&self) -> bool {
        self.jumping
    }

    pub fn is_push_down(&self) -> bool {
        self.push_down
    }
}
