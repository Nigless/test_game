use std::ops::Deref;

use bevy::{
    gltf::{Gltf, GltfMesh, GltfNode},
    prelude::*,
    utils::hashbrown::{HashMap, HashSet},
};
use bevy_rapier3d::geometry::{Collider, ComputedColliderShape};

#[derive(Component)]
pub struct Model {
    pub src: String,
}

impl Model {
    pub fn new<'a>(src: &'a str) -> Self {
        Self {
            src: src.to_owned(),
        }
    }
}

pub struct ModelPlugin;

impl Plugin for ModelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(First, load).add_systems(First, resolve);
    }
}

fn load(server: Res<AssetServer>, mut commands: Commands, models_q: Query<(Entity, &Model)>) {
    for (entity, model) in models_q.iter() {
        let src = model.src.clone();

        commands
            .entity(entity)
            .remove::<Model>()
            .insert(server.load::<Gltf>(src));
    }
}

fn resolve(
    gltf_res: Res<Assets<Gltf>>,
    nodes_res: Res<Assets<GltfNode>>,
    mut gltf_meshes_res: ResMut<Assets<GltfMesh>>,
    mut meshes_res: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    models_q: Query<(Entity, &Handle<Gltf>)>,
) {
    for (entity, gltf) in models_q.iter() {
        gltf_res.get(gltf).map(|gltf| {
            gltf.named_nodes.get("model").map(|node| {
                nodes_res.get(node).map(|node| {
                    resolve_node(
                        &entity,
                        &mut commands,
                        &mut gltf_meshes_res,
                        &mut meshes_res,
                        node,
                    )
                });
            });

            commands.entity(entity).remove::<Handle<Gltf>>();
        });
    }
}

fn resolve_node(
    parent: &Entity,
    commands: &mut Commands,
    gltf_meshes_res: &mut ResMut<Assets<GltfMesh>>,
    meshes_res: &mut ResMut<Assets<Mesh>>,
    node: &GltfNode,
) {
    let mut is_collider = false;

    node.extras.as_ref().map(|extras| {
        if extras.value == "{\"Collider\":true}" {
            is_collider = true
        }
    });

    let mut entity = *parent;

    if !is_collider {
        entity = commands
            .spawn((Name::new("node"), PbrBundle::default()))
            .insert(node.transform.clone())
            .id();

        commands.entity(*parent).add_child(entity);
    }

    node.mesh
        .as_ref()
        .map_or(None, |mesh| gltf_meshes_res.get_mut(mesh))
        .map(|mesh| {
            if mesh.primitives.len() == 1 {
                let primitive = &mesh.primitives[0];

                if is_collider {
                    meshes_res
                        .get(&primitive.mesh)
                        .map_or(None, |mesh| {
                            Collider::from_bevy_mesh(mesh, &ComputedColliderShape::default())
                        })
                        .map(|collider| {
                            commands.entity(entity).insert(collider);
                        });

                    return;
                }

                let mut entity_commands = commands.entity(entity);
                entity_commands.insert(primitive.mesh.clone());

                primitive
                    .material
                    .as_ref()
                    .map(|material| entity_commands.insert(material.clone()));
                return;
            };

            if is_collider {
                panic!("there more than one primitive")
            }

            for primitive in &mesh.primitives {
                let mut primitive_commands =
                    commands.spawn((Name::new("primitive"), primitive.mesh.clone()));

                primitive
                    .material
                    .as_ref()
                    .map(|material| primitive_commands.insert(material.clone()));
                let primitive = primitive_commands.id();
                commands.entity(entity).add_child(primitive);
            }
        });

    node.children
        .iter()
        .for_each(|node| resolve_node(&entity, commands, gltf_meshes_res, meshes_res, node));
}
