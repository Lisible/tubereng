use std::f32::consts::PI;

use tubereng::{
    assets::AssetStore,
    core::Transform,
    ecs::{commands::CommandBuffer, system::ResMut},
    engine::{Engine, EngineBuilder},
    graphics::{
        camera::{ActiveCamera, Camera, FlyCamera},
        geometry::{MeshAsset, MeshDescription, Vertex},
        material::MaterialAsset,
    },
    math::vector::Vector3f,
    winit::WinitTuberRunner,
};

fn main() {
    env_logger::init();
    let engine: Engine = EngineBuilder::new()
        .with_application_title("Procedural terrain")
        .with_setup_system(setup)
        .build();
    pollster::block_on(WinitTuberRunner::run(engine));
}

fn setup(command_buffer: &CommandBuffer, asset_store: ResMut<AssetStore>) {
    let ResMut(mut asset_store) = asset_store;
    command_buffer.insert((
        FlyCamera,
        ActiveCamera,
        Camera::new_perspective(45.0, 800.0 / 600.0, 0.1, 100.0),
        Transform {
            translation: Vector3f::new(0.0, 16.0, 0.0),
            ..Default::default()
        },
    ));
    command_buffer.register_system_set(FlyCamera::system_bundle());

    let ground_mesh_asset = create_ground_mesh();
    let ground_mesh_asset_handle = asset_store.store(ground_mesh_asset);
    let material = asset_store.load::<MaterialAsset>("white.ron").unwrap();
    command_buffer.insert((
        ground_mesh_asset_handle,
        material,
        Transform {
            translation: Vector3f::new(0.0, 0.0, 0.0),
            ..Default::default()
        },
    ));
}

fn create_ground_mesh() -> MeshAsset {
    let mut vertices = vec![];

    let width = 1000;
    let height = 1000;
    let max_elevation = 150.0;
    let terrain_scale = 1.0;

    for j in 0..width {
        for i in 0..height {
            let i = i as f32;
            let j = j as f32;
            let scale = 1000.0;

            let elevations = [
                octave_perlin((i + 0.5) / scale, (j + 0.5) / scale, 6, 4.0) * max_elevation,
                octave_perlin((i + 0.5) / scale, (j + 1.5) / scale, 6, 4.0) * max_elevation,
                octave_perlin((i + 1.5) / scale, (j + 1.5) / scale, 6, 4.0) * max_elevation,
                octave_perlin((i + 1.5) / scale, (j + 0.5) / scale, 6, 4.0) * max_elevation,
            ];

            let color = [
                color_for_elevation(elevations[0]),
                color_for_elevation(elevations[1]),
                color_for_elevation(elevations[2]),
                color_for_elevation(elevations[3]),
            ];
            vertices.push(Vertex {
                position: [i / terrain_scale, elevations[0], j / terrain_scale],
                color: color[0],
                normal: [0.0, 1.0, 0.0],
                texture_coordinates: [0.0, 0.0],
            });
            vertices.push(Vertex {
                position: [i / terrain_scale, elevations[1], (j + 1.0) / terrain_scale],
                color: color[1],
                normal: [0.0, 1.0, 0.0],
                texture_coordinates: [0.0, 1.0],
            });
            vertices.push(Vertex {
                position: [(i + 1.0) / terrain_scale, elevations[3], j / terrain_scale],
                color: color[3],
                normal: [0.0, 1.0, 0.0],
                texture_coordinates: [1.0, 0.0],
            });

            vertices.push(Vertex {
                position: [i / terrain_scale, elevations[1], (j + 1.0) / terrain_scale],
                color: color[1],
                normal: [0.0, 1.0, 0.0],
                texture_coordinates: [0.0, 1.0],
            });
            vertices.push(Vertex {
                position: [
                    (i + 1.0) / terrain_scale,
                    elevations[2],
                    (j + 1.0) / terrain_scale,
                ],
                color: color[2],
                normal: [0.0, 1.0, 0.0],
                texture_coordinates: [1.0, 1.0],
            });
            vertices.push(Vertex {
                position: [(i + 1.0) / terrain_scale, elevations[3], j / terrain_scale],
                color: color[3],
                normal: [0.0, 1.0, 0.0],
                texture_coordinates: [1.0, 0.0],
            });
        }
    }

    MeshAsset {
        mesh_description: MeshDescription {
            vertices,
            indices: None,
        },
    }
}

fn color_for_elevation(elevation: f32) -> [f32; 3] {
    if elevation < 30.0 {
        return [0.0, 0.0, 1.0];
    } else if elevation < 50.0 {
        return [1.0, 1.0, 0.0];
    } else if elevation < 100.0 {
        return [0.0, 1.0, 0.0];
    } else if elevation < 140.0 {
        return [0.2, 0.2, 0.2];
    }

    [1.0, 1.0, 1.0]
}

fn lerp(a: f32, b: f32, w: f32) -> f32 {
    (b - a) * w + a
}

fn random_gradient(ix: i32, iy: i32) -> (f32, f32) {
    let w = 8u32 * 4;
    let s = w / 2u32;
    let mut a: u32 = ix as u32;
    let mut b: u32 = iy as u32;
    a = a.wrapping_mul(3284157443);
    b ^= a << s | a >> (w - s);
    b = b.wrapping_mul(1911520717);
    a ^= b << s | b >> (w - s);
    a = a.wrapping_mul(2048419325);
    let random = a as f32 * (PI / !((!0u32) >> 1) as f32);
    (random.cos(), random.sin())
}

fn dot_grid_gradient(ix: i32, iy: i32, x: f32, y: f32) -> f32 {
    let gradient = random_gradient(ix, iy);
    let dx = x - ix as f32;
    let dy = y - iy as f32;
    dx * gradient.0 + dy * gradient.1
}

fn perlin(x: f32, y: f32) -> f32 {
    let x0 = x.floor() as i32;
    let x1 = x0 + 1;
    let y0 = y.floor() as i32;
    let y1 = y0 + 1;

    let sx = x - x0 as f32;
    let sy = y - y0 as f32;

    let n0 = dot_grid_gradient(x0, y0, x, y);
    let n1 = dot_grid_gradient(x1, y0, x, y);
    let ix0 = lerp(n0, n1, sx);

    let n0 = dot_grid_gradient(x0, y1, x, y);
    let n1 = dot_grid_gradient(x1, y1, x, y);
    let ix1 = lerp(n0, n1, sx);

    lerp(ix0, ix1, sy) * 0.5 + 0.5
}

fn octave_perlin(x: f32, y: f32, octaves: usize, persistence: f32) -> f32 {
    let mut total = 0.0;
    let mut max = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    for _ in 0..octaves {
        total += amplitude * perlin(frequency * x, frequency * y);
        max += amplitude;
        amplitude *= persistence;
        frequency *= 2.0;
    }

    total / max
}
