struct MeshUniform {
    world_transform: mat4x4<f32>,
    inverse_world_transform: mat4x4<f32>,
}

struct CameraUniform {
    position: vec3<f32>,
    view_projection_matrix: mat4x4<f32>,
}

struct LightStorage {
    ambient_light_factor: f32,
    point_light_count: u32,
    point_lights: array<PointLight>,
}

struct PointLight {
    position: vec3<f32>,
    color: vec3<f32>,
    constant: f32,
    linear: f32,
    quadratic: f32
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<uniform> mesh_uniform: MeshUniform;

@group(2) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(2) @binding(1)
var t_sampler: sampler;

@group(3) @binding(0)
var<storage, read> light_storage: LightStorage;


struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) texture_coordinates: vec2<f32>
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) fragment_position: vec3<f32>,
    @location(2) color: vec3<f32>,
    @location(3) normal: vec3<f32>,
    @location(4) texture_coordinates: vec2<f32>
}

@vertex
fn vs_main(
    model: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.fragment_position = (mesh_uniform.world_transform * vec4<f32>(model.position, 1.0)).xyz;    
    out.clip_position = camera.view_projection_matrix * vec4<f32>(out.fragment_position, 1.0);
    out.normal = ((transpose(mesh_uniform.inverse_world_transform)) * vec4<f32>(model.normal, 1.0)).xyz;
    out.texture_coordinates = model.texture_coordinates;
    out.color = model.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let normalized_normal = normalize(in.normal);
    let view_direction = normalize(camera.position - in.fragment_position);
    let texture_sample = vec4<f32>(in.color.xyz, 1.0) * textureSample(t_diffuse, t_sampler, in.texture_coordinates);
    var result = light_storage.ambient_light_factor * texture_sample.xyz;
    for(var i: u32 = 0u; i < light_storage.point_light_count; i++) {
        result += compute_point_light(light_storage.point_lights[i], normalized_normal.xyz, in.fragment_position, view_direction, texture_sample.xyz);
    }

    return vec4<f32>(result, 1.0);
}

fn compute_point_light(light: PointLight, normal: vec3<f32>, fragment_position: vec3<f32>, view_direction: vec3<f32>, texture_sample: vec3<f32>) -> vec3<f32> {
    let light_direction = normalize(light.position - fragment_position);

    // diffuse
    let diffuse = max(dot(normal, light_direction), 0.0);

    // specular
    let shininess = 32.0;
    let reflect_direction = reflect(-light_direction, normal);
    let specular = pow(max(dot(view_direction, reflect_direction), 0.0), shininess);

    // attenuation
    let distance = length(light.position - fragment_position);
    let attenuation = 1.0 / (light.constant + light.linear * distance + light.quadratic * distance * distance);
    
    let diffuse_lighting = light.color * diffuse * texture_sample * attenuation;
    let specular_lighting = light.color * specular * texture_sample * attenuation;
    return diffuse_lighting;
}
