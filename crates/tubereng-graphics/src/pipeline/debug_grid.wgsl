struct CameraUniform {
	position: vec3<f32>,
    view_projection_matrix: mat4x4<f32>,
}

struct InverseCameraUniform {
    inverse_view_projection_matrix: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;
@group(1) @binding(0)
var<uniform> inverse_camera: InverseCameraUniform;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
	@location(1) near_point: vec3<f32>,
	@location(2) far_point: vec3<f32>,
}

var<private> gridPlane: array<vec3<f32>, 6> = array<vec3<f32>, 6>(
	vec3<f32>(-1.0, -1.0, 0.0),
	vec3<f32>(1.0, 1.0, 0.0),
	vec3<f32>(-1.0, 1.0, 0.0),
	vec3<f32>(1.0, 1.0, 0.0),
	vec3<f32>(-1.0, -1.0, 0.0),
	vec3<f32>(1.0, -1.0, 0.0),
);

fn unproject_point(x: f32, y: f32, z: f32) -> vec3<f32> {
	let unprojected_point = inverse_camera.inverse_view_projection_matrix * vec4<f32>(x, y, z, 1.0);
	return unprojected_point.xyz / unprojected_point.w;
}

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
	let point = gridPlane[in_vertex_index].xyz;
	out.near_point = unproject_point(point.x, point.y, 1.0).xyz;
	out.far_point = unproject_point(point.x, point.y, 0.0).xyz;
    out.clip_position = vec4<f32>(point, 1.0);
    return out;
}


struct FragmentOutput {
	@builtin(frag_depth) fragment_depth: f32,
	@location(0) fragment_color: vec4<f32>,
}

// based on https://asliceofrendering.com/scene%20helper/2020/01/05/InfiniteGrid/
fn grid(fragment_position: vec3<f32>, scale: f32, coordinates: vec2<f32>, derivative: vec2<f32>) -> vec4<f32> {
	let grid = abs(fract(coordinates - 0.5) - 0.5) / derivative;
	let line = min(grid.x, grid.y);
	let minimum_z = min(derivative.y, 1.0);
	let minimum_x = min(derivative.x, 1.0);
	var color = vec4<f32>(0.2, 0.2, 0.2, 1.0 - min(line, 1.0));
	if (fragment_position.x > -0.1 * minimum_x && fragment_position.x < 0.1 * minimum_x) {
		color.b = 1.0;
	}
	if (fragment_position.z > -0.1 * minimum_z && fragment_position.z < 0.1 * minimum_z) {
		color.r = 1.0;
	}
	return color;
}

fn compute_fragment_depth(fragment_position: vec3<f32>) -> f32 {
	let projected_position = camera.view_projection_matrix * vec4<f32>(fragment_position, 1.0);
	return (projected_position.z / projected_position.w);
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
	var fragment_output: FragmentOutput;
	let t = -in.near_point.y / (in.far_point.y - in.near_point.y);
	let fragment_position = in.near_point + t * (in.far_point - in.near_point);

    fragment_output.fragment_depth = compute_fragment_depth(fragment_position);
	let scale = 10.0;
	// As of 2023-10-24, the derivative cannot be computed in the grid function
	// See: https://github.com/gfx-rs/naga/issues/2524
	let coordinates = fragment_position.xz * scale;
	let derivative = fwidth(coordinates);
    fragment_output.fragment_color = grid(fragment_position, scale, coordinates, derivative) * f32(t > 0.0);
	return fragment_output;
}
