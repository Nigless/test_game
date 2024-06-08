use std::time::{SystemTime, UNIX_EPOCH};

use bevy::transform::components::Transform;
use bevy::utils::hashbrown::HashMap;

use bevy::prelude::*;

fn date_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[derive(Clone, Reflect, Debug)]
pub struct Keyframe {
    pub timestamp: u64,
    pub value: f32,
}

impl Keyframe {
    pub fn new(timestamp: u64, value: f32) -> Self {
        Self {
            timestamp: timestamp,
            value,
        }
    }
}

impl From<(u64, f32)> for Keyframe {
    fn from((timestamp, value): (u64, f32)) -> Self {
        Keyframe::new(timestamp, value)
    }
}

pub struct Target {
    name: String,
    properties: HashMap<String, Vec<Keyframe>>,
}

impl Target {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            properties: default(),
        }
    }

    pub fn with_property(mut self, name: &str, keyframes: Vec<(u64, f32)>) -> Self {
        self.properties.insert(
            name.to_owned(),
            keyframes.iter().map(|k| Keyframe::from(*k)).collect(),
        );

        self
    }
}

#[derive(Asset, Reflect, Clone)]
#[reflect_value]
pub struct Animation {
    targets: HashMap<String, HashMap<String, Vec<Keyframe>>>,
    duration: u64,
}

impl Animation {
    pub fn new(duration: u64) -> Self {
        Self {
            targets: default(),
            duration: duration,
        }
    }

    pub fn with_duration(mut self, duration: u64) -> Self {
        self.duration = duration;

        self
    }

    pub fn with_target(mut self, target: Target) -> Self {
        self.targets.insert(target.name, target.properties);

        self
    }
}

#[derive(Reflect)]
struct Transition {
    from: f32,
    to: f32,
    duration: u64,
    start_time: u64,
}

#[derive(Reflect)]
pub struct Sequence {
    weight: f32,
    speed: f32,
    playing: bool,
    cursor: u64,
    last_update: u64,
    animation: Handle<Animation>,
    transition: Option<Transition>,
}

impl From<&Handle<Animation>> for Sequence {
    fn from(animation: &Handle<Animation>) -> Self {
        Self::from(animation.clone())
    }
}

impl From<Handle<Animation>> for Sequence {
    fn from(animation: Handle<Animation>) -> Self {
        Self {
            weight: default(),
            speed: 1.0,
            cursor: default(),
            playing: default(),
            last_update: default(),
            transition: default(),
            animation,
        }
    }
}

impl Sequence {
    pub fn with_weight(mut self, weight: f32) -> Self {
        self.weight = weight;

        self
    }

    pub fn set_weight(&mut self, weight: f32) {
        self.weight = weight;
    }

    pub fn set_speed(&mut self, speed: f32) {
        self.update_cursor();
        self.speed = speed;
    }

    pub fn playing(mut self) -> Self {
        self.playing = true;
        self.last_update = date_now();

        self
    }

    fn start(&mut self) {
        self.playing = true;
        self.last_update = date_now();
        self.cursor = 0;
    }

    fn stop(&mut self) {
        self.playing = false
    }

    fn update_transition(&mut self) {
        if self.transition.is_none() {
            return;
        }

        let transition = self.transition.as_mut().unwrap();

        let timestamp = date_now() - transition.start_time;

        if timestamp >= transition.duration {
            self.weight = transition.to;
            self.transition = None;

            return;
        }

        let proportion = timestamp as f32 / transition.duration as f32;

        self.weight = transition.from * (1.0 - proportion) + transition.to * proportion;
    }

    fn update_cursor(&mut self) {
        let date_now = date_now();
        let cursor = date_now - self.last_update;
        self.last_update = date_now;

        self.cursor = (cursor as f64 * self.speed as f64) as u64 + self.cursor;
    }

    fn output(&self, assets: &Res<Assets<Animation>>) -> HashMap<String, HashMap<String, f32>> {
        let animation = assets.get(&self.animation).unwrap();

        let duration = animation.duration;

        let cursor = if duration > 0 {
            (((date_now() - self.last_update) as f64 * self.speed as f64) as u64 + self.cursor)
                % duration
        } else {
            0
        };

        let mut targets: HashMap<String, HashMap<String, f32>> = HashMap::new();

        for (target, properties) in animation.targets.iter() {
            let mut result: HashMap<String, f32> = HashMap::new();

            let mut insert = |property: &str, value: f32| {
                result.insert(property.to_owned(), value * self.weight)
            };

            for (property, keyframes) in properties.iter() {
                let from = keyframes.last();

                if from.is_none() {
                    continue;
                }

                let mut from = from.unwrap();

                if keyframes.len() == 1 {
                    insert(property, from.value);
                    continue;
                }

                let mut to = keyframes.first().unwrap();

                for keyframe in keyframes.iter() {
                    if keyframe.timestamp < cursor {
                        from = keyframe;
                        continue;
                    }

                    to = keyframe;
                    break;
                }

                if to.timestamp == cursor {
                    insert(property, to.value);
                    continue;
                }

                let mut cursor = cursor;
                let mut from = from.clone();
                let mut to = to.clone();

                if to.timestamp < from.timestamp {
                    to.timestamp += animation.duration;
                }

                if cursor < from.timestamp {
                    cursor += animation.duration;
                }

                if from.timestamp != 0 {
                    to.timestamp -= from.timestamp;
                    cursor -= from.timestamp;
                    from.timestamp = 0;
                }

                let proportion = cursor as f32 / to.timestamp as f32;

                let value = from.value * (1.0 - proportion) + to.value * proportion;

                insert(property, value);
            }
            targets.insert(target.to_owned(), result);
        }

        return targets;
    }
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct AnimationSequencer {
    entities: HashMap<String, Entity>,
    sequences: HashMap<String, Sequence>,
}

impl AnimationSequencer {
    pub fn with_animation(mut self, name: &str, animation: &Handle<Animation>) -> Self {
        self.sequences
            .insert(name.to_owned(), Sequence::from(animation));

        self
    }

    pub fn with_sequence(mut self, name: &str, sequence: Sequence) -> Self {
        self.sequences.insert(name.to_owned(), sequence);

        self
    }

    pub fn playing_all(mut self) -> Self {
        self.sequences
            .values_mut()
            .for_each(|sequence| sequence.start());

        self
    }

    pub fn with_weight(mut self, name: &str, weight: f32) -> Self {
        self.sequences.get_mut(name).unwrap().weight = weight;

        self
    }

    pub fn set_weight(&mut self, name: &str, weight: f32) {
        self.sequences.get_mut(name).unwrap().weight = weight
    }

    pub fn with_speed(mut self, name: &str, speed: f32) -> Self {
        self.sequences.get_mut(name).unwrap().speed = speed;

        self
    }

    pub fn set_speed(&mut self, name: &str, speed: f32) {
        self.sequences.get_mut(name).unwrap().set_speed(speed);
    }

    fn get_sequence_mut(&mut self, name: &str) -> Option<&mut Sequence> {
        self.sequences.get_mut(name)
    }

    pub fn set_transition(&mut self, animation: &str, value: f32, duration: u64) {
        let sequence = self.sequences.get_mut(animation).unwrap();
        sequence.transition = Some(Transition {
            from: sequence.weight,
            to: value,
            duration,
            start_time: date_now(),
        })
    }

    fn update_transitions(&mut self) {
        for sequence in self.sequences.values_mut() {
            if sequence.transition.is_none() {
                continue;
            }

            let (from, to, duration, start_time) = sequence
                .transition
                .as_ref()
                .map(|transition| {
                    (
                        transition.from,
                        transition.to,
                        transition.duration,
                        transition.start_time,
                    )
                })
                .unwrap();

            let timestamp = date_now() - start_time;

            if timestamp >= duration {
                sequence.weight = to;
                sequence.transition = None;

                continue;
            }

            let proportion = timestamp as f32 / duration as f32;

            sequence.weight = from * (1.0 - proportion) + to * proportion;
        }
    }

    fn output(&mut self, assets: &Res<Assets<Animation>>) -> HashMap<Entity, HashMap<String, f32>> {
        self.update_transitions();

        let mut result: HashMap<Entity, HashMap<String, f32>> = HashMap::new();

        for (_animation_name, sequence) in self.sequences.iter() {
            if !sequence.playing {
                continue;
            }

            for (target, properties) in sequence.output(&assets).iter() {
                let target = self.entities.get(target).unwrap();

                if result.get(target).is_none() {
                    result.insert(target.to_owned(), HashMap::new());
                }

                let values = result.get_mut(target).unwrap();

                for (property, value) in properties {
                    let value = values
                        .get_mut(property)
                        .map_or(*value, |collector| *collector + *value);
                    values.insert(property.to_owned(), value);
                }
            }
        }

        return result;
    }
}

impl From<&HashMap<String, Handle<Animation>>> for AnimationSequencer {
    fn from(animation_set: &HashMap<String, Handle<Animation>>) -> Self {
        let mut sequences: HashMap<String, Sequence> = HashMap::new();

        for (name, asset) in animation_set.iter() {
            sequences.insert(name.to_owned(), Sequence::from(asset));
        }

        AnimationSequencer {
            entities: default(),
            sequences,
        }
    }
}

#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub enum AnimationSequencerSystems {
    Resolve,
    Update,
}

pub struct AnimationSequencerPlugin;

impl Plugin for AnimationSequencerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Animation>()
            .init_asset::<Animation>()
            .register_asset_reflect::<Animation>()
            .insert_resource(Assets::<Animation>::default())
            .register_type::<AnimationSequencer>()
            .configure_sets(
                PreUpdate,
                (
                    AnimationSequencerSystems::Resolve,
                    AnimationSequencerSystems::Update,
                )
                    .chain(),
            )
            .add_systems(
                PreUpdate,
                (
                    resolve.in_set(AnimationSequencerSystems::Resolve),
                    update.in_set(AnimationSequencerSystems::Update),
                ),
            );
    }
}

fn collect_entities(
    entity: &Entity,
    query: &Query<(Option<&Name>, Option<&Children>)>,
    collector: &mut HashMap<String, Entity>,
) {
    if let Ok((name, children)) = query.get(entity.to_owned()) {
        name.map(|name| {
            collector.insert(name.into(), entity.to_owned());
        });

        children.map(|children| {
            children
                .iter()
                .for_each(|child| collect_entities(child, query, collector));
        });
    }
}

fn resolve(
    mut entity_q: Query<(&mut AnimationSequencer, &Children), Added<AnimationSequencer>>,
    target_q: Query<(Option<&Name>, Option<&Children>)>,
) {
    for (mut sequencer, children) in entity_q.iter_mut() {
        let mut targets: HashMap<String, Entity> = HashMap::new();

        for child in children.iter() {
            collect_entities(child, &target_q, &mut targets)
        }

        sequencer.entities = targets
    }
}

fn update(
    assets: Res<Assets<Animation>>,
    mut entity_q: Query<&mut AnimationSequencer>,
    mut target_q: Query<Option<&mut Transform>, Without<AnimationSequencer>>,
) {
    for mut sequencer in entity_q.iter_mut() {
        if sequencer.entities.is_empty() {
            continue;
        };

        for (entity, properties) in sequencer.output(&assets).iter() {
            if let Ok(mut transform) = target_q.get_mut(entity.to_owned()) {
                for (property, value) in properties.iter() {
                    transform.as_mut().map(|transform| {
                        if property == "transform.scale.x" {
                            transform.scale.x = value.clone();
                        };
                        if property == "transform.scale.y" {
                            transform.scale.y = value.clone();
                        };
                        if property == "transform.scale.z" {
                            transform.scale.z = value.clone();
                        };

                        if property == "transform.translation.x" {
                            transform.translation.x = value.clone();
                        };
                        if property == "transform.translation.y" {
                            transform.translation.y = value.clone();
                        };
                        if property == "transform.translation.z" {
                            transform.translation.z = value.clone();
                        };

                        if property == "transform.rotation.x" {
                            transform.rotation.x = value.clone();
                        };
                        if property == "transform.rotation.y" {
                            transform.rotation.y = value.clone();
                        };
                        if property == "transform.rotation.z" {
                            transform.rotation.z = value.clone();
                        };
                        if property == "transform.rotation.w" {
                            transform.rotation.w = value.clone();
                        }
                    });
                }
            }
        }
    }
}
