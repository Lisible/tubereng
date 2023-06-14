#![warn(clippy::pedantic)]

use tubereng::{engine::EngineBuilder, winit::WinitTuberRunner};

fn main() {
    let engine = EngineBuilder::new()
        .with_application_title("Basic Application")
        .build();
    WinitTuberRunner::run(engine);
}
