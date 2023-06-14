#![warn(clippy::pedantic)]

use tubereng::{ecs::system::EcsCommandBuffer, engine::EngineBuilder, winit::WinitTuberRunner};

fn main() {
    let engine = EngineBuilder::new()
        .with_application_title("Basic Application")
        .with_setup_system(setup)
        .build();
    WinitTuberRunner::run(engine);
}

struct Player;
struct Enemy;
struct Health(i32);

fn setup(ecs_command_buffer: &mut EcsCommandBuffer) {
    ecs_command_buffer.insert((Player, Health(10)));
    ecs_command_buffer.insert((Enemy, Health(5)));
    ecs_command_buffer.insert((Enemy, Health(8)));
}
