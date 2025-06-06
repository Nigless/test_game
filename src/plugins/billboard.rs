use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    render::{
        mesh::MeshVertexBufferLayoutRef,
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
        },
    },
};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct BillboardMaterial {
    #[texture(0)]
    #[sampler(1)]
    texture: Handle<Image>,
}

impl BillboardMaterial {
    pub fn new(texture: Handle<Image>) -> Self {
        Self { texture }
    }
}

const SHADER_PATH: &str = "billboard/billboard.wgsl";

impl Material for BillboardMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER_PATH.into()
    }

    fn vertex_shader() -> ShaderRef {
        SHADER_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(1),
        ])?;

        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}

pub struct BillboardPlugin;

impl Plugin for BillboardPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<BillboardMaterial>::default());
    }
}
