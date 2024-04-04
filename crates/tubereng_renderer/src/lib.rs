#![warn(clippy::pedantic)]

use std::borrow::BorrowMut;

use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawWindowHandle};
use tubereng_ecs::system::ResMut;
use ui_pass::{DrawUiQuadCommand, UiPass};
use wgpu::SurfaceTargetUnsafe;

mod ui_pass;
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

enum DrawCommand {
    DrawUiQuad(DrawUiQuadCommand),
}

pub struct WgpuState<'w> {
    surface: wgpu::Surface<'w>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    _surface_configuration: wgpu::SurfaceConfiguration,
    clear_color: wgpu::Color,
    _window_size: WindowSize,
    _window: RawWindowHandle,
}

pub struct GraphicsState<'w> {
    pub(crate) wgpu_state: WgpuState<'w>,
    commands: Vec<DrawCommand>,
    ui_pass: UiPass,
}

impl<'w> GraphicsState<'w> {
    /// Creates a new `WGPUState`
    ///
    /// # Panics
    ///
    /// Will panic if:
    ///  - The surface cannot be created
    ///  - No adapter is found
    ///  - The device cannot be set up
    ///  - The handle of the window cannot be obtained
    pub async fn new<W>(window: W) -> Self
    where
        W: HasWindowHandle + HasDisplayHandle + std::marker::Send + std::marker::Sync,
    {
        const WINDOW_SIZE: WindowSize = WindowSize {
            width: 800,
            height: 600,
        };

        let mut instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let mut surface = unsafe {
            instance.create_surface_unsafe(
                SurfaceTargetUnsafe::from_window(&window)
                    .expect("Couldn't create SurfaceTargetUnsafe"),
            )
        };

        if surface.is_err() {
            instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
                backends: wgpu::Backends::GL,
                ..Default::default()
            });

            surface = unsafe {
                instance.create_surface_unsafe(
                    SurfaceTargetUnsafe::from_window(&window)
                        .expect("Couldn't create SurfaceTargetUnsafe"),
                )
            };
        }

        let surface = surface.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("No adapter found");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None,
            )
            .await
            .expect("Couldn't setup device");
        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .unwrap_or(surface_capabilities.formats[0]);

        let window_size = WINDOW_SIZE;
        let surface_configuration = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window_size.width,
            height: window_size.height,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_configuration);

        let ui_pass = UiPass::new(&device, &queue);

        GraphicsState {
            wgpu_state: WgpuState {
                surface,
                device,
                queue,
                _surface_configuration: surface_configuration,
                clear_color: wgpu::Color {
                    r: 0.1,
                    g: 0.2,
                    b: 0.3,
                    a: 1.0,
                },
                _window_size: window_size,
                _window: window
                    .window_handle()
                    .expect("Couldn't obtain window handle")
                    .into(),
            },
            commands: vec![],
            ui_pass,
        }
    }

    pub fn draw_ui_quad(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.commands
            .push(DrawCommand::DrawUiQuad(DrawUiQuadCommand {
                x,
                y,
                width,
                height,
            }));
    }
}

pub fn update_clear_color(mut graphics: ResMut<GraphicsState>) {
    graphics.wgpu_state.clear_color.r += 0.0001;
    if graphics.wgpu_state.clear_color.r > 1.0 {
        graphics.wgpu_state.clear_color.r = 0.0;
    }
    graphics.wgpu_state.clear_color.g += 0.0003;
    if graphics.wgpu_state.clear_color.g > 1.0 {
        graphics.wgpu_state.clear_color.g = 0.0;
    }
    graphics.wgpu_state.clear_color.b += 0.0004;
    if graphics.wgpu_state.clear_color.b > 1.0 {
        graphics.wgpu_state.clear_color.b = 0.0;
    }
}

pub fn prepare_frame_system(mut graphics: ResMut<GraphicsState>) {
    let graphics = graphics.borrow_mut();
    let graphics = &mut ***graphics;
    graphics
        .ui_pass
        .prepare(&graphics.wgpu_state, &graphics.commands);
    graphics.commands.clear();
}

/// Renders a frame
///
/// # Panics
///
/// Panics if the surface texture cannot be obtained
pub fn render_frame_system(graphics: ResMut<GraphicsState>) {
    let surface_texture = graphics.wgpu_state.surface.get_current_texture().unwrap();
    let surface_texture_view = surface_texture
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder =
        graphics
            .wgpu_state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("encoder"),
            });

    rpass_clear(
        &mut encoder,
        &surface_texture_view,
        graphics.wgpu_state.clear_color,
    );
    graphics
        .ui_pass
        .execute(&mut encoder, &surface_texture_view);

    graphics
        .wgpu_state
        .queue
        .submit(std::iter::once(encoder.finish()));
    surface_texture.present();
    std::mem::drop(graphics);
}

fn rpass_clear(
    encoder: &mut wgpu::CommandEncoder,
    surface_texture_view: &wgpu::TextureView,
    clear_color: wgpu::Color,
) {
    let _rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("clear_pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: surface_texture_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(clear_color),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
    });
}
