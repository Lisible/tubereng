use std::fmt::Display;

use log::info;
use tubereng::{
    ecs::{
        commands::CommandQueue,
        system::{stages::Update, Res, Q},
    },
    engine::Engine,
    input::{keyboard::Key, InputState},
    winit::WinitTuberRunner,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[derive(Debug)]
struct Player;
#[derive(Debug)]
struct Enemy;

#[derive(Debug)]
#[allow(dead_code)]
struct Position(i32, i32);

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }
    let engine = Engine::builder()
        .with_application_title("basic-app")
        .with_init_system(|queue: &CommandQueue| {
            queue.insert((Player, Position(23, 15)));
            queue.insert((Enemy, Position(2, 5)));
            queue.insert((Enemy, Position(3, 1)));
            queue.register_system(&Update, update_player_position);
            queue.register_system(&Update, print_player_position);
        })
        .build();
    WinitTuberRunner::run(engine).await.unwrap();
}

fn update_player_position(
    input_state: Res<InputState>,
    mut query_player_pos: Q<(&Player, &mut Position)>,
) {
    let is_key_down = |k| input_state.keyboard.is_key_down(k);
    let (_, position) = query_player_pos.first().unwrap();
    if is_key_down(Key::W) {
        position.1 += 1;
    } else if is_key_down(Key::S) {
        position.1 -= 1;
    } else if is_key_down(Key::A) {
        position.0 -= 1;
    } else if is_key_down(Key::D) {
        position.0 += 1;
    }
}

fn print_player_position(mut query_player_pos: Q<(&Player, &Position)>) {
    let (_, position) = query_player_pos.iter().next().unwrap();
    info!("Player position: {position}")
}
