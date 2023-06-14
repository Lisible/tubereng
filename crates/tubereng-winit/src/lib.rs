use tubereng_engine::Engine;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

pub struct WinitTuberRunner;
impl WinitTuberRunner {
    pub fn run(engine: Engine) {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title(engine.application_title())
            .build(&event_loop)
            .unwrap();
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
