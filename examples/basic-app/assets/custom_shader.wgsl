struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) coord: vec2<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32
) -> VertexOutput {
    var vertices = array<vec2<f32>, 3>(
        vec2<f32>(-1., 1.0),
        vec2<f32>(-1., 0.0),
        vec2<f32>(0.5, 1.0),
    );
    var out: VertexOutput;
    out.coord =vertices[in_vertex_index];
    out.clip_position = vec4<f32>(out.coord, 0.0, 1.0);
    return out;
}

const ITERATIONS: i32 = 45;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let c: vec2<f32> = (in.coord + vec2<f32>(-1.5, -1.)) * 1.3;
    var x: f32 = 0.;
    var y: f32 = 0.;
    var i: i32 = 0;
    
    for (; i < ITERATIONS; i = i + 1) {
        if (x*x + y*y > 4.) {
            break;
        }
        let xtemp: f32 = (x * x) - (y * y) + c.x;
        y = 2. * x * y + c.y;
        x = xtemp;
    }

    let frac: f32 = f32(i) / f32(ITERATIONS);
    return vec4<f32>(frac * 5., frac * 1., frac * 3., 1.0);
}