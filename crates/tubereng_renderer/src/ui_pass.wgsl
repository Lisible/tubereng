struct CommonUniforms {
	projection_matrix: mat4x4<f32>,
}

struct VertexInput {
	@location(0) position: vec3<f32>,
	@location(1) color: vec3<f32>,
	@location(2) texture_coordinates: vec2<f32>,
}

struct VertexOutput {
	@builtin(position) position: vec4<f32>,
	@location(0) color: vec3<f32>,
	@location(1) texture_coordinates: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> u_common: CommonUniforms;
@group(1) @binding(0)
var texture: texture_2d<f32>;
@group(1) @binding(1)
var texture_sampler: sampler;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
	var out: VertexOutput;
	out.position = u_common.projection_matrix * vec4<f32>(in.position, 1.0);
	out.color = in.color;
	out.texture_coordinates = in.texture_coordinates;
	return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	let sample = textureSample(texture, texture_sampler, vec2<f32>(in.texture_coordinates.x, in.texture_coordinates.y));	
	return vec4<f32>(in.color, 1.0) * sample;
}
