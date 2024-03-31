use tubereng::{
    ecs::commands::{CommandQueue, InsertEntity},
    engine::Engine,
    winit::{WinitError, WinitTuberRunner},
};

#[derive(Debug)]
struct Player;
#[derive(Debug)]
struct Enemy;

#[derive(Debug)]
#[allow(dead_code)]
struct Position(i32, i32);

fn main() -> Result<(), WinitError> {
    env_logger::init();
    let engine = Engine::builder()
        .with_application_title("basic-app")
        .with_init_system(|queue: &CommandQueue| {
            queue.insert((Player, Position(23, 15)));
            queue.insert((Enemy, Position(2, 5)));
            queue.insert((Enemy, Position(3, 1)));
            queue.register_system(|| println!("Some system"));
        })
        .build();
    WinitTuberRunner::run(engine)
}
