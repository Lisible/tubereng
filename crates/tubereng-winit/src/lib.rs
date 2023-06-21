use log::info;
use tubereng_engine::Engine;
use tubereng_graphics::{Renderer, WindowSize};
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub struct WinitTuberRunner;
impl WinitTuberRunner {
    pub async fn run(mut engine: Engine) {
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

        let renderer = Renderer::new(window).await;
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
                }
                _ => (),
            }
        });
    }
}
