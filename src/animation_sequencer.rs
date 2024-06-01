use std::default;
use std::ops::Deref;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bevy::ecs::reflect;
use bevy::scene::ron::value;
use bevy::transform::commands;
use bevy::transform::components::Transform;
use bevy::utils::hashbrown::HashMap;
use bevy::{animation, prelude::*};

fn date_now() -> Duration {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
}

#[derive(Clone, Reflect)]
pub struct Keyframe {
    pub timestamp: Duration,
    pub value: f32,
}

impl Keyframe {
    pub fn new(timestamp: u64, value: f32) -> Self {
        Self {
            timestamp: Duration::from_millis(timestamp),
            value,
        }
    }
}

#[derive(Asset, Reflect, Clone)]
#[reflect_value]
pub struct Animation {
    nodes: HashMap<String, HashMap<String, Vec<Keyframe>>>,
    duration: Duration,
}

impl Animation {
    pub fn new(duration: u64) -> Self {
        Self {
            nodes: default(),
            duration: Duration::from_millis(duration),
        }
    }

    pub fn with_duration(mut self, duration: u64) -> Self {
        self.duration = Duration::from_millis(duration);

        self
    }

    pub fn with_node(mut self, name: &str) -> Self {
        self.nodes.insert(name.to_owned(), HashMap::new());

        self
    }

    pub fn with_property(mut self, name: &str, keyframes: Vec<Keyframe>) -> Self {
        self.nodes
            .values_mut()
            .last()
            .unwrap()
            .insert(name.to_owned(), keyframes);

        self
    }
}

#[derive(Reflect)]
struct Transition {
    from: f32,
    to: f32,
    duration: Duration,
    start_time: Duration,
}

#[derive(Reflect)]
pub struct Sequence {
    weight: f32,
    playing: bool,
    start_time: Duration,
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
            weight: 1.0,
            playing: default(),
            start_time: default(),
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

    pub fn playing(mut self) -> Self {
        self.playing = true;
        self.start_time = date_now();

        self
    }

    fn start(&mut self) {
        self.playing = true;
        self.start_time = date_now()
    }

    fn stop(&mut self) {
        self.playing = false
    }

    fn output(&self, assets: &Res<Assets<Animation>>) -> HashMap<String, HashMap<String, f32>> {
        let animation = assets.get(&self.animation).unwrap();

        let time_now = date_now();
        let mut timestamp = time_now - self.start_time;

        if timestamp > animation.duration {
            timestamp = Duration::from_millis(if animation.duration.as_millis() > 0 {
                timestamp.as_millis() % animation.duration.as_millis()
            } else {
                0
            } as u64);
        }

        let mut nods: HashMap<String, HashMap<String, f32>> = HashMap::new();

        for (node, properties) in animation.nodes.iter() {
            let mut result: HashMap<String, f32> = HashMap::new();

            let mut insert = |property: &str, value: f32| {
                result.insert(property.to_owned(), value * self.weight)
            };

            for (property, keyframes) in properties.iter() {
                let from = keyframes.first();

                if from.is_none() {
                    continue;
                }

                let mut from = from.unwrap();

                if keyframes.len() == 1 {
                    insert(property, from.value);
                    continue;
                }

                let mut to = from;

                for keyframe in keyframes.iter() {
                    if keyframe.timestamp < timestamp {
                        from = keyframe;
                    } else {
                        to = keyframe;
                        break;
                    }
                }

                if to.timestamp == timestamp {
                    insert(property, to.value);
                    continue;
                }

                let mut from = from.clone();
                let mut to = to.clone();

                if to.timestamp < from.timestamp {
                    to.timestamp = animation.duration + to.timestamp;
                }

                if !from.timestamp.is_zero() {
                    from.timestamp = Duration::ZERO;
                    to.timestamp -= from.timestamp;
                }

                let proportion = timestamp.as_millis() as f32 / to.timestamp.as_millis() as f32;

                let value = from.value * (1.0 - proportion) + to.value * proportion;

                insert(property, value);
            }
            nods.insert(node.to_owned(), result);
        }

        return nods;
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

    pub fn set_weight(&mut self, name: &str, weight: f32) {
        self.sequences.get_mut(name).unwrap().weight = weight
    }

    fn get_sequence_mut(&mut self, name: &str) -> Option<&mut Sequence> {
        self.sequences.get_mut(name)
    }

    pub fn add_transition(&mut self, animation: &str, value: f32, duration: u64) {
        let sequence = self.sequences.get_mut(animation).unwrap();
        sequence.transition = Some(Transition {
            from: sequence.weight,
            to: value,
            duration: Duration::from_millis(duration),
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

            let proportion = timestamp.as_millis() as f32 / duration.as_millis() as f32;

            sequence.weight = from * (1.0 - proportion) + to * proportion;
        }
    }

    fn output(&mut self, assets: &Res<Assets<Animation>>) -> HashMap<Entity, HashMap<String, f32>> {
        self.update_transitions();

        let mut nods: HashMap<Entity, HashMap<String, f32>> = HashMap::new();

        for (animation_name, sequence) in self.sequences.iter() {
            if !sequence.playing {
                continue;
            }

            for (node, properties) in sequence.output(&assets).iter() {
                let node = self.entities.get(node).unwrap();

                let mut result: HashMap<String, f32> = HashMap::new();

                for (property, value) in properties {
                    let value: f32 = nods
                        .get(node)
                        .map_or(None, |node| node.get(property))
                        .map_or(*value, |collector| *collector + *value);

                    result.insert(property.to_owned(), value);
                }
                nods.insert(node.to_owned(), result);
            }
        }

        return nods;
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

pub struct AnimationSequencerPlugin;

impl Plugin for AnimationSequencerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Animation>()
            .init_asset::<Animation>()
            .register_asset_reflect::<Animation>()
            .insert_resource(Assets::<Animation>::default())
            .register_type::<AnimationSequencer>()
            .add_systems(First, resolve)
            .add_systems(Update, update);
    }
}

fn collect_entities(
    entity: &Entity,
    query: &Query<(&Name, Option<&Children>)>,
    collector: &mut HashMap<String, Entity>,
) {
    if let Ok((name, children)) = query.get(entity.to_owned()) {
        collector.insert(name.into(), entity.to_owned());

        children.map(|children| {
            children
                .iter()
                .for_each(|child| collect_entities(child, query, collector));
        });
    }
}

fn resolve(
    mut entity_q: Query<(&mut AnimationSequencer, &Children), Added<AnimationSequencer>>,
    node_q: Query<(&Name, Option<&Children>)>,
) {
    for (mut sequencer, children) in entity_q.iter_mut() {
        let mut nodes: HashMap<String, Entity> = HashMap::new();

        for child in children.iter() {
            collect_entities(child, &node_q, &mut nodes)
        }

        sequencer.entities = nodes
    }
}

fn update(
    assets: Res<Assets<Animation>>,
    mut entity_q: Query<&mut AnimationSequencer>,
    mut node_q: Query<Option<&mut Transform>, Without<AnimationSequencer>>,
) {
    for mut sequencer in entity_q.iter_mut() {
        for (entity, properties) in sequencer.output(&assets).iter() {
            if let Ok(mut transform) = node_q.get_mut(entity.to_owned()) {
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
                    });
                }
            }
        }
    }
}
