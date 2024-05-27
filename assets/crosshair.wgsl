// This shader draws a circle with a given input color
#import bevy_ui::ui_vertex_output::UiVertexOutput

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
   // the UVs are now adjusted around the middle of the rect.
   let uv = in.uv * 2.0 - 1.0;

   // circle alpha, the higher the power the harsher the falloff.
   let value = (1.0 - sqrt(dot(uv, uv))) * 2.0;


   var color = 0.0;
   
   if value > 1.0 {
      color = 1.0;
   };

   return vec4<f32>(color, color, color, value);
}