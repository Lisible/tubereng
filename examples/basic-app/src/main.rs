#![warn(clippy::pedantic)]
#![allow(clippy::needless_pass_by_value)]

use std::f32::consts::PI;

use tubereng::{
    core::Transform,
    ecs::{commands::CommandBuffer, query::Q},
    engine::EngineBuilder,
    graphics::{
        camera::{ActiveCamera, Camera},
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

fn setup(command_buffer: &CommandBuffer) {
    command_buffer.insert((
        ActiveCamera,
        Camera::new_perspective(45.0, 800.0 / 600.0, 0.1, 100.0),
        Transform {
            translation: Vector3f::new(0.0, 0.0, 5.0),
            ..Default::default()
        },
    ));
    command_buffer.insert((
        Cube,
        Transform {
            translation: Vector3f::new(-1.0, 0.0, 0.0),
            scale: Vector3f::new(0.5, 0.5, 0.5),
            rotation: Quaternion::from_axis_angle(&Vector3f::new(0.0, 1.0, 0.0), PI / 6.0),
        },
    ));

    command_buffer.insert((
        Cube,
        Transform {
            translation: Vector3f::new(1.0, 0.0, 0.0),
            scale: Vector3f::new(0.5, 0.5, 0.5),
            rotation: Quaternion::from_axis_angle(&Vector3f::new(0.0, 1.0, 0.0), PI / 6.0),
        },
    ));

    command_buffer.register_system(rotate_cubes);
}

fn rotate_cubes(cube_query: Q<(&Cube, &mut Transform)>) {
    for (_, mut transform) in cube_query.iter() {
        transform.rotation = transform.rotation.clone()
            * Quaternion::from_axis_angle(&Vector3f::new(0.0, 1.0, 0.5), 0.01);
    }
}
