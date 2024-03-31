#![warn(clippy::pedantic)]

use tubereng_engine::Engine;
use tubereng_input::{keyboard::Key, Input};
use winit::{
    dpi::PhysicalSize,
    error::{EventLoopError, OsError},
    event::{DeviceEvent, Event, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
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
    pub fn run(mut engine: Engine) -> Result<(), WinitError> {
        let event_loop = EventLoop::new().map_err(WinitError::EventLoopCreationFailed)?;
        let window = WindowBuilder::new()
            .with_title(engine.application_title())
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
                    engine.update();
                    window.request_redraw();
                }
                Event::WindowEvent {
                    event: WindowEvent::RedrawRequested,
                    ..
                } => {
                    engine.render();
                }
                Event::DeviceEvent {
                    event: DeviceEvent::MouseMotion { delta },
                    ..
                } => engine.on_input(Input::MouseMotion(delta)),
                Event::WindowEvent {
                    event:
                        WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    state,
                                    physical_key: PhysicalKey::Code(virtual_keycode),
                                    ..
                                },
                            ..
                        },
                    ..
                } => match state {
                    winit::event::ElementState::Pressed => {
                        engine.on_input(Input::KeyDown(WinitKeyCode(virtual_keycode).into()));
                    }

                    winit::event::ElementState::Released => {
                        engine.on_input(Input::KeyUp(WinitKeyCode(virtual_keycode).into()));
                    }
                },
                _ => {}
            })
            .map_err(WinitError::EventLoopRunningFailed)?;

        Ok(())
    }
}

struct WinitKeyCode(KeyCode);
impl From<WinitKeyCode> for Key {
    fn from(value: WinitKeyCode) -> Self {
        let virtual_key_code = value.0;
        match virtual_key_code {
            KeyCode::Escape => Key::Escape,
            KeyCode::Space => Key::Space,
            KeyCode::ArrowUp => Key::ArrowUp,
            KeyCode::ArrowDown => Key::ArrowDown,
            KeyCode::ArrowLeft => Key::ArrowLeft,
            KeyCode::ArrowRight => Key::ArrowRight,
            KeyCode::KeyA => Key::A,
            KeyCode::KeyB => Key::B,
            KeyCode::KeyC => Key::C,
            KeyCode::KeyD => Key::D,
            KeyCode::KeyE => Key::E,
            KeyCode::KeyF => Key::F,
            KeyCode::KeyG => Key::G,
            KeyCode::KeyH => Key::H,
            KeyCode::KeyI => Key::I,
            KeyCode::KeyJ => Key::J,
            KeyCode::KeyK => Key::K,
            KeyCode::KeyL => Key::L,
            KeyCode::KeyM => Key::M,
            KeyCode::KeyN => Key::N,
            KeyCode::KeyO => Key::O,
            KeyCode::KeyP => Key::P,
            KeyCode::KeyQ => Key::Q,
            KeyCode::KeyR => Key::R,
            KeyCode::KeyS => Key::S,
            KeyCode::KeyT => Key::T,
            KeyCode::KeyU => Key::U,
            KeyCode::KeyV => Key::V,
            KeyCode::KeyW => Key::W,
            KeyCode::KeyX => Key::X,
            KeyCode::KeyY => Key::Y,
            KeyCode::KeyZ => Key::Z,
            _ => Key::Unknown,
        }
    }
}
