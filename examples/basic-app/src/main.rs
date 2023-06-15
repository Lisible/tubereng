#![warn(clippy::pedantic)]

use tubereng::{ecs::commands::CommandBuffer, engine::EngineBuilder, winit::WinitTuberRunner};

fn main() {
    env_logger::init();
    let engine = EngineBuilder::new()
        .with_application_title("Basic Application")
        .with_setup_system(setup)
        .build();
    WinitTuberRunner::run(engine);
}

#[derive(Debug)]
struct Player;
#[derive(Debug)]
struct Enemy;
#[derive(Debug)]
struct Health(i32);

fn setup(command_buffer: &mut CommandBuffer) {
    command_buffer.insert((Player, Health(10)));
    command_buffer.insert((Enemy, Health(5)));
    command_buffer.insert((Enemy, Health(8)));
    command_buffer.register_system(log_player_health);
}

fn log_player_health() {
    println!("10");
}
