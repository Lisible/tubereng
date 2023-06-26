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
        Cube,
        Transform {
            translation: Vector3f::new(0.0, 0.0, 0.0),
            scale: Vector3f::new(1.0, 1.0, 1.0),
            rotation: Quaternion::new(1.0, Vector3f::new(0.0, 0.0, 0.0)),
        },
    ));
}
