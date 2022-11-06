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

#[derive(Component)]
pub struct Sprite;

pub struct SpritePlugin;

impl Plugin for SpritePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(resolve).add_system(rotate);
    }
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
            .spawn()
            .insert(Sprite)
            .insert(Transform::default())
            .insert(GlobalTransform::default())
            .insert(Visibility::default())
            .insert(ComputedVisibility::default())
            .insert(meshes.add(Mesh::from(shape::Quad {
                size: Vec2::new(1.0, 1.0),
                flip: false,
            })))
            .insert(materials.add(StandardMaterial {
                base_color_texture: Some(server.load(sprite.src.as_str())),
                // perceptual_roughness: 1.0,
                // metallic: 0.0,
                // reflectance: 0.0,
                // unlit: true,
                // double_sided: false,
                alpha_mode: AlphaMode::Mask(0.5),
                ..Default::default()
            }))
            .id();

        commands
            .entity(entity)
            .remove::<WithSprite>()
            .push_children(&[child]);
    }
}

fn rotate(
    mut sprites_q: Query<&mut Transform, With<Sprite>>,
    mut camera_q: Query<&mut Transform, (With<Camera>, Without<Sprite>)>,
) {
    if let Err(_) = camera_q.get_single() {
        return;
    }
    let camera_transform = camera_q.single_mut();

    for mut sprite_transform in sprites_q.iter_mut() {
        sprite_transform.rotation = camera_transform.rotation
    }
}
