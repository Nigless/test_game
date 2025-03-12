#import bevy_pbr::{
   mesh_view_bindings::view,
   mesh_functions::get_world_from_local
}

@group(2) @binding(0)
var billboard_texture: texture_2d<f32>;
@group(2) @binding(1)
var billboard_sampler: sampler;

struct Vertex {
   @builtin(instance_index) instance_index: u32,
   @location(0) position: vec3<f32>,
   @location(1) uv: vec2<f32>,

};
struct VertexOutput {
   @builtin(position) position: vec4<f32>,
   @location(0) uv: vec2<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {

   let camera_right = normalize(vec3<f32>(view.clip_from_world.x.x, view.clip_from_world.y.x, view.clip_from_world.z.x));

   let camera_up = normalize(vec3<f32>(view.clip_from_world.x.y, view.clip_from_world.y.y, view.clip_from_world.z.y));

   let world_space = camera_right * vertex.position.x + camera_up * vertex.position.y;
   let position = view.clip_from_world * get_world_from_local(vertex.instance_index) * vec4<f32>(world_space, 1.);


   var out: VertexOutput;
   out.position = position;
   out.uv = vertex.uv;

   return out;
}

struct FragmentInput {
   @location(0) uv: vec2<f32>,
};

@fragment
fn fragment(fragment: FragmentInput) -> @location(0) vec4<f32> {
   let color = textureSample(billboard_texture, billboard_sampler, fragment.uv);

   return color;

}