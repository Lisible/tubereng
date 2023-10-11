#![warn(clippy::pedantic)]
#![allow(clippy::needless_pass_by_value)]

use tubereng::{
    assets::{AssetHandle, AssetStore},
    core::Transform,
    ecs::{
        commands::CommandBuffer,
        event::EventWriter,
        query::Q,
        system::{Res, ResMut},
    },
    engine::{Engine, EngineBuilder, ExitRequest},
    graphics::{
        camera::{ActiveCamera, Camera, FlyCamera},
        geometry::{MeshAsset, MeshDescription, Vertex},
        light::PointLight,
        material::{MaterialAsset, PbrMaterialAsset, ShaderMaterialAsset},
        pipeline::default_pipeline::DefaultRenderPipelineSettings,
    },
    input::{keyboard::Key, InputState},
    math::{quaternion::Quaternion, vector::Vector3f},
    winit::WinitTuberRunner,
};

fn main() {
    env_logger::init();
    let engine: Engine = EngineBuilder::new()
        .with_application_title("Basic Application")
        .with_setup_system(setup)
        .with_render_pipeline_settings(DefaultRenderPipelineSettings {
            render_debug_grid: true,
        })
        .build();
    pollster::block_on(WinitTuberRunner::run(engine));
}

fn setup(command_buffer: &CommandBuffer, asset_store: ResMut<AssetStore>) {
    command_buffer.insert((
        FlyCamera,
        ActiveCamera,
        Camera::new_perspective(45.0, 800.0 / 600.0, 0.1, 100.0),
        Transform {
            translation: Vector3f::new(0.0, 1.0, 0.0),
            rotation: Quaternion::from_axis_angle(
                &Vector3f::new(1.0, 0.0, 0.0),
                -std::f32::consts::PI / 6.0,
            ),
            ..Default::default()
        },
    ));
    command_buffer.register_system_set(FlyCamera::system_bundle());

    let ResMut(mut asset_store) = asset_store;
    let material = asset_store.load::<MaterialAsset>("material.ron").unwrap();
    let material2 = asset_store.load::<MaterialAsset>("material2.ron").unwrap();
    let material3 =
        asset_store.store::<MaterialAsset>(MaterialAsset::ShaderMaterial(ShaderMaterialAsset {
            shader: "custom_shader.wgsl".into(),
        }));
    let cone_model = asset_store.load::<MeshAsset>("cone.obj").unwrap();
    let cube_model = asset_store.load::<MeshAsset>("cube.obj").unwrap();
    let light_model = asset_store.load::<MeshAsset>("lightbulb.obj").unwrap();
    let light_material = asset_store
        .load::<MaterialAsset>("lightbulb_material.ron")
        .unwrap();

    let grid_mesh_asset = create_grid_mesh(100, 100);
    let grid_mesh = asset_store.store::<MeshAsset>(grid_mesh_asset);
    let grass_material = asset_store
        .load::<MaterialAsset>("grass_material.ron")
        .unwrap();

    command_buffer.insert((
        grid_mesh,
        material3,
        Transform {
            translation: Vector3f::new(2.0, 1.0, -2.0),
            ..Default::default()
        },
    ));
    command_buffer.insert((
        cone_model,
        material,
        Transform {
            translation: Vector3f::new(0.0, 0.0, -5.0),
            ..Default::default()
        },
    ));
    command_buffer.insert((
        cube_model,
        material2,
        Transform {
            translation: Vector3f::new(1.0, 0.0, 1.0),
            ..Default::default()
        },
    ));

    command_buffer.insert_resource(LightAssets {
        model: light_model,
        material: light_material,
    });
    command_buffer.register_system(spawn_light_at_camera_position);
    command_buffer.register_system(exit);
}

fn create_grid_mesh(width: usize, height: usize) -> MeshAsset {
    let mut vertices = vec![];
    let mut indices = vec![];

    let mut v = 0;
    for j in 0..width {
        for i in 0..height {
            let i = i as f32;
            let j = j as f32;
            vertices.push(Vertex {
                position: [i, j.sin() / 10.0, j],
                color: [1.0, 1.0, 1.0],
                normal: [0.0, 1.0, 0.0],
                texture_coordinates: [0.0, 0.0],
            });
            vertices.push(Vertex {
                position: [i, (j + 1.0).sin() / 10.0, j + 1.0],
                color: [1.0, 1.0, 1.0],
                normal: [0.0, 1.0, 0.0],
                texture_coordinates: [0.0, 1.0],
            });
            vertices.push(Vertex {
                position: [i + 1.0, (j + 1.0).sin() / 10.0, j + 1.0],
                color: [1.0, 1.0, 1.0],
                normal: [0.0, 1.0, 0.0],
                texture_coordinates: [1.0, 1.0],
            });
            vertices.push(Vertex {
                position: [i + 1.0, j.sin() / 10.0, j],
                color: [1.0, 1.0, 1.0],
                normal: [0.0, 1.0, 0.0],
                texture_coordinates: [1.0, 0.0],
            });
            indices.extend_from_slice(&[v + 1, v + 3, v, v + 1, v + 2, v + 3]);
            v += 4;
        }
    }

    MeshAsset {
        mesh_description: MeshDescription {
            vertices,
            indices: Some(indices),
        },
    }
}

fn exit(exit_request_writer: EventWriter<ExitRequest>, input: Res<InputState>) {
    let Res(input) = input;
    if input.keyboard.is_key_down(Key::Escape) {
        exit_request_writer.write(ExitRequest);
    }
}

fn spawn_light_at_camera_position(
    command_buffer: &CommandBuffer,
    camera_query: Q<(&Camera, &Transform)>,
    light_assets: Res<LightAssets>,
    input: Res<InputState>,
) {
    let Res(light_assets) = light_assets;
    let (_, camera_transform) = camera_query.iter().next().unwrap();

    let mut light_transform = camera_transform.clone();
    light_transform.scale = Vector3f::new(0.1, 0.1, 0.1);

    let Res(input) = input;
    if input.keyboard.is_key_down(Key::E) {
        command_buffer.insert((
            PointLight::default(),
            light_assets.model,
            light_assets.material,
            light_transform,
        ));
    }
}

#[derive(Debug)]
struct LightAssets {
    model: AssetHandle<MeshAsset>,
    material: AssetHandle<MaterialAsset>,
}
