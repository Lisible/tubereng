use log::info;
use tubereng_engine::Engine;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

pub struct WinitTuberRunner;
impl WinitTuberRunner {
    pub fn run(mut engine: Engine) {
        info!("Engine starting up...");
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title(engine.application_title())
            .build(&event_loop)
            .unwrap();
        info!(
            "Created window with title \"{}\"",
            engine.application_title()
        );

        info!("Running setup system...");
        engine.run_setup_system();

        info!("Starting main loop...");
        event_loop.run(move |event, _, control_flow| {
            control_flow.set_poll();
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    control_flow.set_exit();
                }
                Event::MainEventsCleared => {
                    window.request_redraw();
                }
                Event::RedrawRequested(_) => {}
                _ => (),
            }
        });
    }
}
