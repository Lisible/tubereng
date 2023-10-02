#![warn(clippy::pedantic)]
#![allow(clippy::needless_pass_by_value)]

use std::f32::consts::PI;

use tubereng::{
    assets::{AssetHandle, AssetStore},
    core::Transform,
    ecs::{
        commands::CommandBuffer,
        query::Q,
        system::{Res, ResMut},
    },
    engine::{Engine, EngineBuilder},
    graphics::{
        camera::{ActiveCamera, Camera, FlyCamera},
        geometry::ModelAsset,
        light::PointLight,
        material::MaterialAsset,
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
            rotation: Quaternion::from_axis_angle(&Vector3f::new(1.0, 0.0, 0.0), -PI / 6.0),
            ..Default::default()
        },
    ));
    command_buffer.register_system_set(FlyCamera::system_bundle());

    let ResMut(mut asset_store) = asset_store;
    let material = asset_store.load::<MaterialAsset>("material.ron").unwrap();
    let material2 = asset_store.load::<MaterialAsset>("material2.ron").unwrap();
    let cone_model = asset_store.load::<ModelAsset>("cone.obj").unwrap();
    let cube_model = asset_store.load::<ModelAsset>("cube.obj").unwrap();
    let light_model = asset_store.load::<ModelAsset>("lightbulb.obj").unwrap();
    let light_material = asset_store
        .load::<MaterialAsset>("lightbulb_material.ron")
        .unwrap();

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
    model: AssetHandle<ModelAsset>,
    material: AssetHandle<MaterialAsset>,
}
