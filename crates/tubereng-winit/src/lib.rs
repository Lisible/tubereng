#![warn(clippy::pedantic)]

use log::info;
use tubereng_engine::Engine;
use tubereng_graphics::{pipeline::RenderPipeline, Renderer, WindowSize};
use tubereng_input::{keyboard::Key, Input};
use winit::{
    event::{DeviceEvent, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub struct WinitTuberRunner;
impl WinitTuberRunner {
    // TODO return a Result and detail when this might panic
    /// Runs the engine
    ///
    /// # Panics
    ///
    /// This might panic if an error occurs during the window initialization
    /// Or during the engine execution
    pub async fn run<R>(mut engine: Engine<R>)
    where
        R: 'static + RenderPipeline,
    {
        info!("Engine starting up...");
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title(engine.application_title())
            .build(&event_loop)
            .unwrap();

        #[cfg(target_arch = "wasm32")]
        {
            use winit::dpi::PhysicalSize;
            window.set_inner_size(PhysicalSize::new(800, 600));

            use winit::platform::web::WindowExtWebSys;
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let dst = doc.get_element_by_id("wasm-example")?;
                    let canvas = web_sys::Element::from(window.canvas());
                    dst.append_child(&canvas).ok()?;
                    Some(())
                })
                .expect("Couldn't append canvas to document body.");
        }

        info!(
            "Created window with title \"{}\"",
            engine.application_title()
        );

        let renderer = Renderer::new(engine.render_pipeline_settings(), window).await;
        engine.initialize_renderer(renderer);

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
                Event::WindowEvent {
                    event: WindowEvent::Resized(size),
                    ..
                } => {
                    engine.resize(WindowSize::new(size.width, size.height));
                }
                Event::WindowEvent {
                    event: WindowEvent::ScaleFactorChanged { new_inner_size, .. },
                    ..
                } => {
                    engine.resize(WindowSize::new(new_inner_size.width, new_inner_size.height));
                }
                Event::MainEventsCleared => {
                    engine.update();
                    engine.render();
                    engine.clear_last_frame_inputs();
                }
                Event::DeviceEvent {
                    event: DeviceEvent::MouseMotion { delta },
                    ..
                } => engine.on_input(Input::MouseMotion(delta)),
                Event::WindowEvent {
                    event:
                        WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state,
                                    virtual_keycode: Some(virtual_keycode),
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
                _ => (),
            }
        });
    }
}

struct WinitKeyCode(VirtualKeyCode);
impl From<WinitKeyCode> for Key {
    fn from(value: WinitKeyCode) -> Self {
        let virtual_key_code = value.0;
        match virtual_key_code {
            VirtualKeyCode::Escape => Key::Escape,
            VirtualKeyCode::Return => Key::Return,
            VirtualKeyCode::LShift => Key::LShift,
            VirtualKeyCode::RShift => Key::RShift,
            VirtualKeyCode::LControl => Key::LControl,
            VirtualKeyCode::RControl => Key::RControl,
            VirtualKeyCode::Back => Key::Backspace,
            VirtualKeyCode::Space => Key::Space,
            VirtualKeyCode::Up => Key::ArrowUp,
            VirtualKeyCode::Down => Key::ArrowDown,
            VirtualKeyCode::Left => Key::ArrowLeft,
            VirtualKeyCode::Right => Key::ArrowRight,
            VirtualKeyCode::A => Key::A,
            VirtualKeyCode::B => Key::B,
            VirtualKeyCode::C => Key::C,
            VirtualKeyCode::D => Key::D,
            VirtualKeyCode::E => Key::E,
            VirtualKeyCode::F => Key::F,
            VirtualKeyCode::G => Key::G,
            VirtualKeyCode::H => Key::H,
            VirtualKeyCode::I => Key::I,
            VirtualKeyCode::J => Key::J,
            VirtualKeyCode::K => Key::K,
            VirtualKeyCode::L => Key::L,
            VirtualKeyCode::M => Key::M,
            VirtualKeyCode::N => Key::N,
            VirtualKeyCode::O => Key::O,
            VirtualKeyCode::P => Key::P,
            VirtualKeyCode::Q => Key::Q,
            VirtualKeyCode::R => Key::R,
            VirtualKeyCode::S => Key::S,
            VirtualKeyCode::T => Key::T,
            VirtualKeyCode::U => Key::U,
            VirtualKeyCode::V => Key::V,
            VirtualKeyCode::W => Key::W,
            VirtualKeyCode::X => Key::X,
            VirtualKeyCode::Y => Key::Y,
            VirtualKeyCode::Z => Key::Z,
            _ => Key::Unknown,
        }
    }
}
