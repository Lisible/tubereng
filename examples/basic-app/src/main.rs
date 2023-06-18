#![warn(clippy::pedantic)]
#![allow(clippy::needless_pass_by_value)]

use tubereng::{
    ecs::{commands::CommandBuffer, query::Q},
    engine::EngineBuilder,
    winit::WinitTuberRunner,
};

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

fn setup(command_buffer: &CommandBuffer) {
    command_buffer.insert((Player, Health(10)));
    command_buffer.insert((Player, Health(9)));
    command_buffer.insert((Player, Health(4)));
    command_buffer.insert((Enemy, Health(5)));
    command_buffer.insert((Enemy, Health(8)));
    command_buffer.register_system(log_player_health);
}

fn log_player_health(player_healths: Q<(&Player, &mut Health)>, healths: Q<(&mut Health,)>) {
    for (_, health) in player_healths.iter() {
        println!("Player health: {}", health.0);
    }
    for (health,) in healths.iter() {
        println!("Entity health: {}", health.0);
    }
}
