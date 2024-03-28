#![warn(clippy::pedantic)]

use winit::{
    dpi::PhysicalSize,
    error::{EventLoopError, OsError},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[derive(Debug)]
pub enum WinitError {
    EventLoopCreationFailed(EventLoopError),
    EventLoopRunningFailed(EventLoopError),
    WindowCreationFailed(OsError),
}

pub struct WinitTuberRunner;
impl WinitTuberRunner {
    /// Starts the application using a winit window.
    ///
    /// # Errors
    ///
    /// Will return [`Err`] if the event loop cannot be created or run, or if
    /// the window cannot be created.
    pub fn run() -> Result<(), WinitError> {
        let event_loop = EventLoop::new().map_err(WinitError::EventLoopCreationFailed)?;
        let window = WindowBuilder::new()
            .with_title("tuber-application")
            .with_resizable(false)
            .with_inner_size(PhysicalSize::new(800, 600))
            .build(&event_loop)
            .map_err(WinitError::WindowCreationFailed)?;
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop
            .run(move |event, elwt| match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    elwt.exit();
                }
                Event::AboutToWait => {
                    window.request_redraw();
                }
                _ => {}
            })
            .map_err(WinitError::EventLoopRunningFailed)?;

        Ok(())
    }
}
