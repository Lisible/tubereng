use tubereng::{
    ecs::{commands::CommandQueue, system::Res},
    engine::Engine,
    input::InputState,
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
            queue.register_system(|input_state: Res<InputState>| {
                if input_state
                    .keyboard
                    .is_key_down(tubereng::input::keyboard::Key::A)
                {
                    println!("Key A down");
                } else {
                    println!("Key A up");
                }
            })
        })
        .build();
    WinitTuberRunner::run(engine)
}
