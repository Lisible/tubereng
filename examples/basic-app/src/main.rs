use tubereng::{
    engine::{Engine, EngineConfiguration},
    winit::{WinitError, WinitTuberRunner},
};

fn main() -> Result<(), WinitError> {
    let engine = Engine::new(&EngineConfiguration {
        application_title: "basic-app",
    });
    WinitTuberRunner::run(engine)
}
