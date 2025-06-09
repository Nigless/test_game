use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bevy::asset::ReflectAsset;
use bevy::utils::hashbrown::hash_map::Iter;
use bevy::utils::HashMap;
use bevy::{ecs::reflect, prelude::*, state::commands};
use bevy_rapier3d::parry::simba::scalar::SupersetOf;
use uuid::Uuid;

use crate::library::date_now;

#[derive(Reflect)]
pub struct Timer {
    repeated: bool,
    duration: Duration,
    started_at: u128,
    paused_at: Option<u128>,
    finished: bool,
    subscribers: Vec<Entity>,
}

impl Timer {
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            repeated: false,
            finished: false,
            started_at: date_now(),
            paused_at: None,
            subscribers: default(),
        }
    }

    pub fn repeated(mut self) -> Self {
        self.repeated = true;
        self
    }

    pub fn with_subscriber(mut self, entity: Entity) -> Self {
        self.subscribers.push(entity);
        self
    }

    pub fn play(&mut self) {
        let Some(paused_at) = self.paused_at else {
            return;
        };

        self.started_at += date_now() - paused_at;

        self.paused_at = None;
    }

    pub fn reset(&mut self) {
        self.started_at = date_now();

        self.paused_at = None;

        self.finished = false;
    }

    pub fn stop(&mut self) {
        self.started_at = date_now();

        self.paused_at = None;

        self.finished = true;
    }

    pub fn pause(&mut self) {
        self.paused_at = Some(date_now());
    }

    fn elapsed(&mut self) -> bool {
        if self.paused_at.is_some() {
            return false;
        }

        if self.finished {
            return false;
        }

        let time_now = date_now();

        let is_time_up = time_now - self.started_at > self.duration.as_millis();

        if is_time_up {
            self.started_at = time_now
        }

        if self.repeated {
            return is_time_up;
        };

        self.finished = is_time_up;

        self.finished
    }
}

#[derive(Clone, Copy, Reflect, PartialEq, Debug)]
pub struct TimerLink(Uuid);

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct TimerCollection {
    list: HashMap<Uuid, Timer>,
}

impl TimerCollection {
    pub fn ids(&self) -> impl Iterator<Item = TimerLink> + '_ {
        self.list.keys().map(|key| TimerLink(*key))
    }

    pub fn get_mut(&mut self, link: &TimerLink) -> Option<&mut Timer> {
        self.list.get_mut(&link.0)
    }

    pub fn add(&mut self, timer: Timer) -> TimerLink {
        let uuid = Uuid::new_v4();

        self.list.insert(uuid, timer);

        TimerLink(uuid)
    }

    pub fn remove(&mut self, link: &TimerLink) -> Option<Timer> {
        self.list.remove(&link.0)
    }
}

#[derive(Event)]
pub struct TimerElapsedEvent(pub TimerLink);

pub struct TimerPlugin;

impl Plugin for TimerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(First, update)
            .register_type::<TimerCollection>()
            .register_type::<u128>()
            .init_resource::<TimerCollection>();
    }
}

fn update(mut timers: ResMut<TimerCollection>, mut commands: Commands) {
    for link in timers.ids().collect::<Vec<_>>() {
        let timer = timers.get_mut(&link).unwrap();

        if timer.finished {
            continue;
        }

        let targets = timer.subscribers.clone();

        if timer.elapsed() {
            if targets.is_empty() {
                commands.trigger(TimerElapsedEvent(link));
                continue;
            }

            commands.trigger_targets(TimerElapsedEvent(link), targets);
        }
    }
}
