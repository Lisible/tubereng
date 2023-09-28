#![warn(clippy::pedantic)]
#![allow(clippy::needless_pass_by_value)]

use std::f32::consts::PI;

use tubereng::{
    assets::{AssetHandle, AssetStore},
    core::Transform,
    ecs::{commands::CommandBuffer, query::Q, system::ResMut},
    engine::EngineBuilder,
    graphics::{
        camera::{ActiveCamera, Camera},
        geometry::ModelAsset,
        material::{Material, MaterialAsset},
        Cube,
    },
    math::{quaternion::Quaternion, vector::Vector3f},
    winit::WinitTuberRunner,
};

fn main() {
    env_logger::init();
    let engine = EngineBuilder::new()
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
            translation: Vector3f::new(0.0, 0.0, 5.0),
            ..Default::default()
        },
    ));

    let ResMut(mut asset_store) = asset_store;
    let material = asset_store.load::<MaterialAsset>("material.ron").unwrap();
    let material2 = asset_store.load::<MaterialAsset>("material2.ron").unwrap();
    let cone_model = asset_store.load::<ModelAsset>("cone.obj").unwrap();
    let cube_model = asset_store.load::<ModelAsset>("cube.obj").unwrap();

    command_buffer.insert((
        cone_model,
        material,
        Transform {
            translation: Vector3f::new(0.0, 0.0, 0.0),
            scale: Vector3f::new(0.5, 0.5, 0.5),
            rotation: Quaternion::from_axis_angle(&Vector3f::new(0.0, 1.0, 0.0), PI / 6.0),
        },
    ));
    command_buffer.insert((
        cone_model,
        material2,
        Transform {
            translation: Vector3f::new(0.0, 0.0, 0.0),
            scale: Vector3f::new(0.5, 0.5, 0.5),
            rotation: Quaternion::from_axis_angle(&Vector3f::new(0.0, 1.0, 0.0), PI / 6.0),
        },
    ));

    command_buffer.insert((
        cube_model,
        material,
        Transform {
            translation: Vector3f::new(2.0, 0.0, 0.0),
            scale: Vector3f::new(0.5, 0.5, 0.5),
            rotation: Quaternion::from_axis_angle(&Vector3f::new(0.0, 1.0, 0.0), PI / 6.0),
        },
    ));

    command_buffer.register_system(rotate_models);
}

fn rotate_models(cube_query: Q<(&AssetHandle<ModelAsset>, &mut Transform)>) {
    for (_, mut transform) in cube_query.iter() {
        transform.rotation = transform.rotation.clone()
            * Quaternion::from_axis_angle(&Vector3f::new(0.0, 1.0, 0.5), 0.01);
    }
}
