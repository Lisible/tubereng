#![warn(clippy::pedantic)]

use std::{borrow::BorrowMut, sync::Arc};

use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawWindowHandle};
use render_graph::{RenderGraph, RenderPass};
use tubereng_ecs::{
    system::{stages, Res, ResMut},
    Ecs, Storage,
};
use wgpu::SurfaceTargetUnsafe;

pub mod camera;
pub mod material;
mod mesh;
mod pass_2d;
pub mod render_graph;
pub mod sprite;
pub mod texture;

pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

pub struct WgpuState<'w> {
    surface: wgpu::Surface<'w>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_configuration: wgpu::SurfaceConfiguration,
    window_size: WindowSize,
    _window: RawWindowHandle,
}

pub struct GraphicsState<'w> {
    pub(crate) wgpu_state: WgpuState<'w>,
    pub(crate) texture_cache: texture::Cache,
    material_bind_group_layout: wgpu::BindGroupLayout,
    placeholder_material_id: Option<material::Id>,
    pub(crate) material_cache: material::Cache,
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

        let surface = Self::create_surface(&mut instance, &window);

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

        let material_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("material_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        GraphicsState {
            wgpu_state: WgpuState {
                surface,
                device,
                queue,
                surface_configuration,
                window_size,
                _window: window
                    .window_handle()
                    .expect("Couldn't obtain window handle")
                    .into(),
            },
            texture_cache: texture::Cache::new(),
            material_cache: material::Cache::new(),
            placeholder_material_id: None,
            material_bind_group_layout,
        }
    }

    pub fn window_size(&self) -> &WindowSize {
        &self.wgpu_state.window_size
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.wgpu_state.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.wgpu_state.queue
    }

    pub fn surface_texture_format(&self) -> wgpu::TextureFormat {
        self.wgpu_state.surface_configuration.format
    }

    fn create_surface<W>(instance: &mut wgpu::Instance, window: &W) -> wgpu::Surface<'w>
    where
        W: HasWindowHandle + HasDisplayHandle + std::marker::Send + std::marker::Sync,
    {
        let mut surface = unsafe {
            instance.create_surface_unsafe(
                SurfaceTargetUnsafe::from_window(window)
                    .expect("Couldn't create SurfaceTargetUnsafe"),
            )
        };

        if surface.is_err() {
            *instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
                backends: wgpu::Backends::GL,
                ..Default::default()
            });

            surface = unsafe {
                instance.create_surface_unsafe(
                    SurfaceTargetUnsafe::from_window(window)
                        .expect("Couldn't create SurfaceTargetUnsafe"),
                )
            };
        }

        surface.unwrap()
    }

    pub fn load_texture(&mut self, descriptor: &texture::Descriptor) -> texture::Id {
        let texture_size = wgpu::Extent3d {
            width: descriptor.width,
            height: descriptor.height,
            depth_or_array_layers: 1,
        };

        // TODO add texture path as label
        let texture = self
            .wgpu_state
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });

        self.wgpu_state.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            descriptor.data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * descriptor.width),
                rows_per_image: Some(descriptor.height),
            },
            texture_size,
        );

        let texture_info = texture::Info {
            width: descriptor.width,
            height: descriptor.height,
        };

        self.texture_cache.insert(texture_info, texture)
    }

    pub fn load_material(&mut self, descriptor: &material::Descriptor) -> material::Id {
        let device = &self.wgpu_state.device;
        let base_color_texture = self.texture_cache.get(descriptor.base_color);
        let base_color_texture_view =
            base_color_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let base_color_texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.material_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&base_color_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&base_color_texture_sampler),
                },
            ],
        });

        self.material_cache
            .insert(material::Material { bind_group })
    }
}

pub struct FrameRenderingContext {
    pub surface_texture: Option<wgpu::SurfaceTexture>,
    pub surface_texture_view: Option<wgpu::TextureView>,
    pub encoder: Option<wgpu::CommandEncoder>,
}

pub async fn renderer_init<W>(
    ecs: &mut Ecs,
    window: Arc<W>,
    placeholder_texture: &texture::Descriptor<'_>,
) where
    W: HasWindowHandle + HasDisplayHandle + std::marker::Send + std::marker::Sync,
{
    let mut gfx = GraphicsState::new(window).await;
    let placeholder_texture_id = gfx.load_texture(placeholder_texture);
    let placeholder_material_id = gfx.load_material(&material::Descriptor {
        base_color: placeholder_texture_id,
        region: texture::Rect {
            x: 0.0,
            y: 0.0,
            width: 16.0,
            height: 16.0,
        },
    });
    gfx.placeholder_material_id = Some(placeholder_material_id);

    ecs.insert_resource(gfx);
    ecs.insert_resource(RenderGraph::new());
    ecs.insert_resource(FrameRenderingContext {
        surface_texture: None,
        surface_texture_view: None,
        encoder: None,
    });

    ecs.register_system(&stages::Render, begin_frame_system);
    ecs.register_system(&stages::Render, add_clear_pass_system);
    ecs.register_system(&stages::Render, pass_2d::add_pass_system);
    ecs.register_system(&stages::FinalizeRender, prepare_passes_system);
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

fn prepare_passes_system(mut graph: ResMut<RenderGraph>, storage: &Storage) {
    graph.prepare(storage);
}

/// Renders a frame
///
/// # Panics
///
/// Panics if the surface texture cannot be obtained
fn finish_frame_system(
    mut graphics: ResMut<GraphicsState>,
    mut frame_ctx: ResMut<FrameRenderingContext>,
    graph: Res<RenderGraph>,
    storage: &Storage,
) {
    let mut encoder = frame_ctx.encoder.take().unwrap();
    let surface_texture_view = frame_ctx.surface_texture_view.take().unwrap();
    graph.execute(&mut graphics, &mut encoder, &surface_texture_view, storage);
    graphics
        .wgpu_state
        .queue
        .submit(std::iter::once(encoder.finish()));

    let surface_texture = frame_ctx.surface_texture.take().unwrap();
    surface_texture.present();
    std::mem::drop(graphics);
    std::mem::drop(graph);
}

fn add_clear_pass_system(mut graph: ResMut<RenderGraph>) {
    graph.add_pass(ClearPass);
}

pub struct ClearPass;
impl RenderPass for ClearPass {
    fn prepare(&mut self, _storage: &Storage) {}
    fn execute(
        &self,
        _gfx: &mut GraphicsState,
        encoder: &mut wgpu::CommandEncoder,
        surface_texture_view: &wgpu::TextureView,
        _storage: &Storage,
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
