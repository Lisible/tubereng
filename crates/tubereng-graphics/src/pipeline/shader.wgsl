struct MeshUniform {
    world_transform: mat4x4<f32>,
}

struct CameraUniform {
    view_projection_matrix: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<uniform> mesh_uniform: MeshUniform;

@group(2) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(2) @binding(1)
var t_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) texture_coordinates: vec2<f32>
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) texture_coordinates: vec2<f32>
}

@vertex
fn vs_main(
    model: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_projection_matrix * mesh_uniform.world_transform * vec4<f32>(model.position, 1.0);
    out.texture_coordinates = model.texture_coordinates;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, t_sampler, in.texture_coordinates);
}
