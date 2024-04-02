use std::fmt::Display;

use tubereng::{
    ecs::{
        commands::CommandQueue,
        system::{stages::Update, Res, Q},
    },
    engine::Engine,
    input::{keyboard::Key, InputState},
    winit::{WinitError, WinitTuberRunner},
};

#[derive(Debug)]
struct Player;
#[derive(Debug)]
struct Enemy;

#[derive(Debug)]
#[allow(dead_code)]
struct Position(i32, i32);

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "({}, {})", self.0, self.1)
    }
}

fn main() -> Result<(), WinitError> {
    env_logger::init();
    let engine = Engine::builder()
        .with_application_title("basic-app")
        .with_init_system(|queue: &CommandQueue| {
            queue.insert((Player, Position(23, 15)));
            queue.insert((Enemy, Position(2, 5)));
            queue.insert((Enemy, Position(3, 1)));
            queue.register_system::<Update, _, _>(update_player_position);
            queue.register_system::<Update, _, _>(print_player_position);
        })
        .build();
    WinitTuberRunner::run(engine)
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
    println!("Player position: {position}")
}
