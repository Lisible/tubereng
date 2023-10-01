#![warn(clippy::pedantic)]
#![allow(clippy::needless_pass_by_value)]

use std::f32::consts::PI;

use tubereng::{
    assets::{AssetHandle, AssetStore},
    core::Transform,
    ecs::{commands::CommandBuffer, query::Q, system::ResMut},
    engine::{Engine, EngineBuilder},
    graphics::{
        camera::{ActiveCamera, Camera},
        geometry::ModelAsset,
        material::MaterialAsset,
    },
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
        ActiveCamera,
        Camera::new_perspective(45.0, 800.0 / 600.0, 0.1, 100.0),
        Transform {
            translation: Vector3f::new(0.0, 1.0, 0.0),
            rotation: Quaternion::from_axis_angle(&Vector3f::new(1.0, 0.0, 0.0), -PI / 6.0),
            ..Default::default()
        },
    ));

    let ResMut(mut asset_store) = asset_store;
    let material = asset_store.load::<MaterialAsset>("material.ron").unwrap();
    let material2 = asset_store.load::<MaterialAsset>("material2.ron").unwrap();
    let cone_model = asset_store.load::<ModelAsset>("cone.obj").unwrap();
    let cube_model = asset_store.load::<ModelAsset>("cube.obj").unwrap();

    // command_buffer.insert((
    //     cone_model,
    //     material,
    //     Transform {
    //         translation: Vector3f::new(0.0, 0.0, -5.0),
    //         ..Default::default()
    //     },
    // ));

    command_buffer.register_system(rotate_camera);
}

fn rotate_camera(camera_query: Q<(&Camera, &mut Transform)>) {
    for (_, mut transform) in camera_query.iter() {
        transform.translation.z -= 0.01;
        transform.translation.y -= 0.01;
    }
}
