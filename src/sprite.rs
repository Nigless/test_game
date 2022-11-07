use crate::camera::Camera;
use bevy::prelude::*;

#[derive(Component)]
pub struct WithSprite {
    pub src: String,
}

impl WithSprite {
    pub fn new<'a>(src: &'a str) -> Self {
        Self {
            src: src.to_owned(),
        }
    }
}

#[derive(Component, Default)]
pub struct Sprite;

#[derive(Component, Default)]
pub struct SpriteBody;

pub struct SpritePlugin;

impl Plugin for SpritePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::PreUpdate, resolve)
            .add_system(rotate);
    }
}

#[derive(Bundle, Default)]
struct BndSpriteBody {
    sprite: SpriteBody,
    visibility: Visibility,
    computed_visibility: ComputedVisibility,
    transform: Transform,
    global_transform: GlobalTransform,
}

fn resolve(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    server: Res<AssetServer>,
    mut commands: Commands,
    sprites_q: Query<(Entity, &WithSprite)>,
) {
    for (entity, sprite) in sprites_q.iter() {
        let child = commands
            .spawn_bundle(BndSpriteBody::default())
            .insert_bundle((
                materials.add(StandardMaterial {
                    base_color_texture: Some(server.load(sprite.src.as_str())),

                    alpha_mode: AlphaMode::Mask(0.5),
                    ..Default::default()
                }),
                meshes.add(Mesh::from(shape::Quad {
                    size: Vec2::new(1.0, 1.0),
                    flip: false,
                })),
            ))
            .id();

        commands
            .entity(entity)
            .remove::<WithSprite>()
            .insert(Sprite)
            .push_children(&[child]);
    }
}

fn rotate(
    mut sprites_q: Query<&mut Transform, With<SpriteBody>>,
    camera_q: Query<&mut Transform, (With<Camera>, Without<SpriteBody>)>,
) {
    let camera_transform = match camera_q.get_single() {
        Ok(c) => c,
        Err(_) => return,
    };

    for mut sprite_transform in sprites_q.iter_mut() {
        sprite_transform.rotation = camera_transform.rotation
    }
}
