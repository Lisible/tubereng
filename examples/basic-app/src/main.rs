#![warn(clippy::pedantic)]

use tubereng::{engine::EngineBuilder, winit::WinitTuberRunner};

fn main() {
    let engine = EngineBuilder::new()
        .with_application_title("Basic Application")
        .with_setup_system(setup)
        .build();
    WinitTuberRunner::run(engine);
}

fn setup() {
    println!("Setting up");
}
