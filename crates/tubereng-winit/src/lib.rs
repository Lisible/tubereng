#![warn(clippy::pedantic)]

#[cfg(feature = "egui")]
use egui::{FontDefinitions, Style};
#[cfg(feature = "egui")]
use egui_winit_platform::{Platform, PlatformDescriptor};
use log::info;
use std::sync::Arc;
use std::time::Instant;
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
    #[allow(clippy::too_many_lines)]
    pub async fn run<R>(mut engine: Engine<R>)
    where
        R: 'static + RenderPipeline,
    {
        info!("Engine starting up...");
        let event_loop = EventLoop::new();
        let window = Arc::new(
            WindowBuilder::new()
                .with_title(engine.application_title())
                .build(&event_loop)
                .unwrap(),
        );
        window
            .set_cursor_grab(winit::window::CursorGrabMode::Confined)
            .unwrap();
        window.set_cursor_visible(false);

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

        #[cfg(feature = "egui")]
        let mut platform = Platform::new(PlatformDescriptor {
            physical_width: window.inner_size().width,
            physical_height: window.inner_size().height,
            scale_factor: window.scale_factor(),
            font_definitions: FontDefinitions::default(),
            style: Style::default(),
        });
        let renderer = Renderer::new(engine.render_pipeline_settings(), window.clone()).await;
        engine.initialize_renderer(renderer);
        engine.run_setup_system();

        info!("Starting main loop...");
        let mut last_frame_start_instant = Instant::now();
        let start_time = Instant::now();
        event_loop.run(move |event, _, control_flow| {
            control_flow.set_poll();
            #[cfg(feature = "egui")]
            platform.handle_event(&event);
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => engine.exit(),
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
                    #[cfg(feature = "egui")]
                    platform.update_time(start_time.elapsed().as_secs_f64());
                    let frame_start_instant = Instant::now();
                    let delta_time = (frame_start_instant - last_frame_start_instant).as_secs_f32();
                    if engine.should_exit() {
                        control_flow.set_exit();
                    }

                    engine.begin_frame();
                    #[cfg(feature = "egui")]
                    platform.begin_frame();

                    engine.update(
                        delta_time,
                        #[cfg(feature = "egui")]
                        platform.context(),
                    );

                    #[cfg(feature = "egui")]
                    let egui_output = platform.end_frame(Some(&window));

                    engine.prepare_render();

                    #[cfg(feature = "egui")]
                    engine.render(platform.context(), egui_output);
                    #[cfg(not(feature = "egui"))]
                    engine.render();

                    engine.clear_last_frame_inputs();
                    last_frame_start_instant = frame_start_instant;
                    profiling::finish_frame!();
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
