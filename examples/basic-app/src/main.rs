#![warn(clippy::pedantic)]
#![allow(clippy::needless_pass_by_value)]

use tubereng::{
    core::Transform,
    ecs::{
        commands::CommandBuffer,
        query::Q,
        system::{Res, ResMut},
    },
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
            translation: Vector3f::new(0.0, 0.0, 2.0),
            ..Default::default()
        },
    ));
    command_buffer.insert((
        Cube,
        Transform {
            translation: Vector3f::new(0.0, 0.0, 0.0),
            scale: Vector3f::new(1.0, 1.0, 1.0),
            rotation: Quaternion::new(1.0, Vector3f::new(0.0, 0.0, 0.0)),
        },
    ));

    command_buffer.register_system(update_frame_number);
    command_buffer.register_system(update_camera);
    command_buffer.insert_resource(FrameNumber(0));
}

fn update_frame_number(frame_number: ResMut<FrameNumber>) {
    let ResMut(mut frame_number) = frame_number;
    frame_number.0 += 1;
}

fn update_camera(camera_query: Q<(&ActiveCamera, &mut Transform)>, frame_number: Res<FrameNumber>) {
    let Res(frame_number) = frame_number;
    let (_, mut camera_transform) = camera_query.iter().next().unwrap();
    camera_transform.translation.z = 2.0 + (frame_number.0 as f64 / 20f64).sin() as f32;
}

struct FrameNumber(pub u64);
