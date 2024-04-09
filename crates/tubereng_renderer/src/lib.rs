#![warn(clippy::pedantic)]

use std::{borrow::BorrowMut, collections::HashMap, sync::Arc};

use draw_triangle_pass::{create_draw_triangle_pass_pipeline, DrawTrianglePass};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawWindowHandle};
use render_graph::{RenderGraph, RenderPass};
use tubereng_ecs::{
    system::{stages, Res, ResMut},
    Ecs,
};
use wgpu::SurfaceTargetUnsafe;

mod draw_triangle_pass;
pub mod render_graph;

pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

pub struct WgpuState<'w> {
    surface: wgpu::Surface<'w>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    _surface_configuration: wgpu::SurfaceConfiguration,
    _window_size: WindowSize,
    _window: RawWindowHandle,
}

pub struct GraphicsState<'w> {
    pub(crate) wgpu_state: WgpuState<'w>,
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

        GraphicsState {
            wgpu_state: WgpuState {
                surface,
                device,
                queue,
                _surface_configuration: surface_configuration,
                _window_size: window_size,
                _window: window
                    .window_handle()
                    .expect("Couldn't obtain window handle")
                    .into(),
            },
        }
    }
}

pub struct FrameRenderingContext {
    pub surface_texture: Option<wgpu::SurfaceTexture>,
    pub surface_texture_view: Option<wgpu::TextureView>,
    pub encoder: Option<wgpu::CommandEncoder>,
}

pub async fn renderer_init<W>(ecs: &mut Ecs, window: Arc<W>)
where
    W: HasWindowHandle + HasDisplayHandle + std::marker::Send + std::marker::Sync,
{
    let gfx = GraphicsState::new(window).await;
    let mut pipelines = RenderPipelines::new();
    let draw_triangle_pass_pipeline = create_draw_triangle_pass_pipeline(
        &gfx.wgpu_state.device,
        wgpu::TextureFormat::Bgra8UnormSrgb,
    );
    pipelines.insert("draw_triangle_pass_pipeline", draw_triangle_pass_pipeline);

    ecs.insert_resource(gfx);
    ecs.insert_resource(RenderGraph::new());
    ecs.insert_resource(FrameRenderingContext {
        surface_texture: None,
        surface_texture_view: None,
        encoder: None,
    });

    ecs.insert_resource(pipelines);
    ecs.register_system(&stages::Render, begin_frame_system);
    ecs.register_system(&stages::Render, add_clear_pass);
    ecs.register_system(&stages::Render, add_draw_triangle_pass);
    ecs.register_system(&stages::FinalizeRender, finish_frame_system);
}

fn begin_frame_system(
    mut graphics: ResMut<GraphicsState>,
    mut frame_ctx: ResMut<FrameRenderingContext>,
    mut graph: ResMut<RenderGraph>,
) {
    let graphics = graphics.borrow_mut();
    let surface_texture = graphics.wgpu_state.surface.get_current_texture().unwrap();
    let surface_texture_view = surface_texture
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());
    let encoder =
        graphics
            .wgpu_state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("encoder"),
            });

    frame_ctx.surface_texture = Some(surface_texture);
    frame_ctx.surface_texture_view = Some(surface_texture_view);
    frame_ctx.encoder = Some(encoder);

    graph.clear();
}

/// Renders a frame
///
/// # Panics
///
/// Panics if the surface texture cannot be obtained
fn finish_frame_system(
    graphics: ResMut<GraphicsState>,
    mut frame_ctx: ResMut<FrameRenderingContext>,
    graph: Res<RenderGraph>,
    pipelines: Res<RenderPipelines>,
) {
    let mut encoder = frame_ctx.encoder.take().unwrap();
    let surface_texture_view = frame_ctx.surface_texture_view.take().unwrap();
    graph.execute(&pipelines, &mut encoder, &surface_texture_view);
    graphics
        .wgpu_state
        .queue
        .submit(std::iter::once(encoder.finish()));

    let surface_texture = frame_ctx.surface_texture.take().unwrap();
    surface_texture.present();
    std::mem::drop(graphics);
    std::mem::drop(graph);
    std::mem::drop(pipelines);
}

fn add_clear_pass(mut graph: ResMut<RenderGraph>) {
    graph.add_pass(ClearPass);
}

fn add_draw_triangle_pass(mut graph: ResMut<RenderGraph>) {
    graph.add_pass(DrawTrianglePass);
}

pub struct ClearPass;
impl RenderPass for ClearPass {
    fn prepare(&mut self) {}
    fn execute(
        &self,
        _pipelines: &RenderPipelines,
        encoder: &mut wgpu::CommandEncoder,
        surface_texture_view: &wgpu::TextureView,
    ) {
        let _rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("clear_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: surface_texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
    }
}

pub struct RenderPipelines {
    pipelines: HashMap<String, wgpu::RenderPipeline>,
}

impl RenderPipelines {
    #[must_use]
    pub fn new() -> Self {
        Self {
            pipelines: HashMap::new(),
        }
    }

    pub fn insert<S>(&mut self, identifier: S, pipeline: wgpu::RenderPipeline)
    where
        S: Into<String>,
    {
        self.pipelines.insert(identifier.into(), pipeline);
    }

    #[must_use]
    pub fn get(&self, identifier: &str) -> &wgpu::RenderPipeline {
        &self.pipelines[identifier]
    }
}

impl Default for RenderPipelines {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Color {
    r: f32,
    g: f32,
    b: f32,
}

impl Color {
    pub const BLACK: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
    };
    pub const WHITE: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
    };

    #[must_use]
    pub fn new(r: f32, g: f32, b: f32) -> Color {
        Color { r, g, b }
    }
}

impl From<&Color> for [f32; 3] {
    fn from(value: &Color) -> Self {
        [value.r, value.g, value.b]
    }
}
