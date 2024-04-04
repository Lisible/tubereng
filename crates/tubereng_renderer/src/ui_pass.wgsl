struct CommonUniforms {
	projection_matrix: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> u_common: CommonUniforms;

@vertex
fn vs_main(@location(0) position: vec3<f32>) -> @builtin(position) vec4<f32> {
	return u_common.projection_matrix * vec4<f32>(position, 1.0);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
	return vec4<f32>(0.02, 0.02, 0.02, 1.0);
}
