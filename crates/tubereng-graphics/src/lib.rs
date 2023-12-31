#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use geometry::ModelCache;
use material::{MaterialAsset, MaterialCache};
use pipeline::RenderPipeline;
use std::{collections::HashMap, future::Future, sync::Arc};
use texture::TextureCache;
use tubereng_assets::{AssetHandle, AssetStore};
use tubereng_ecs::{entity::EntityStore, relationship::RelationshipStore};
use winit::{dpi::PhysicalSize, window::Window};

pub type Result<T> = std::result::Result<T, GraphicsError>;
#[derive(Debug)]
pub enum GraphicsError {
    ModelAssetNotFound,
    MaterialAssetNotFound,
    TextureAssetNotFound,
    InvalidMesh,
    AssetError(tubereng_assets::AssetError),
}

#[derive(Debug)]
pub struct Cube {
    pub material: AssetHandle<MaterialAsset>,
}

pub mod camera;
pub mod color;
pub mod geometry;
pub mod light;
pub mod material;
pub mod pipeline;
pub mod render_graph;
pub mod texture;

#[derive(Clone, Copy)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

impl WindowSize {
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

pub struct Renderer<R>
where
    R: RenderPipeline,
{
    window: Arc<Window>,
    rendering_context: RenderingContext,
    pipeline: R,

    #[cfg(feature = "egui")]
    egui_pass: egui_wgpu_backend::RenderPass,
}

impl<R> Renderer<R>
where
    R: RenderPipeline,
{
    pub async fn new(
        render_pipeline_settings: &R::RenderPipelineSettings,
        window: Arc<Window>,
    ) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Safety: WGPURenderer owns both the window and the surface, so the window will live as
        // long as the surface
        let surface =
            unsafe { instance.create_surface(&*window) }.expect("Surface creation failed");
        let adapter = Self::request_adapter(&instance, &surface)
            .await
            .expect("Adapter not found");

        let (device, queue) = Self::request_device(&adapter)
            .await
            .expect("Couldn't request device");

        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = Self::get_compatible_surface_format(&surface_capabilities);
        let surface_configuration = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &surface_configuration);
        let mut shader_modules = HashMap::new();
        let vertex_buffers = vec![];
        let index_buffers = vec![];
        let material_cache = MaterialCache::new(&device);

        let mut texture_cache = TextureCache::new();
        let pipeline = R::new(
            render_pipeline_settings,
            &device,
            &surface_configuration,
            &mut texture_cache,
            &mut shader_modules,
        );

        #[cfg(feature = "egui")]
        let egui_pass =
            egui_wgpu_backend::RenderPass::new(&device, surface_configuration.format, 1);

        let rendering_context = RenderingContext {
            device,
            queue,
            draw_commands: vec![],
            vertex_buffers,
            index_buffers,
            texture_cache,
            material_cache,
            model_cache: ModelCache::new(),
            size,
            surface,
            surface_configuration,
            pipelines: HashMap::new(),
            shader_modules,
        };

        Self {
            window,
            rendering_context,
            pipeline,

            #[cfg(feature = "egui")]
            egui_pass,
        }
    }

    fn request_adapter(
        instance: &wgpu::Instance,
        surface: &wgpu::Surface,
    ) -> impl Future<Output = Option<wgpu::Adapter>> {
        instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(surface),
        })
    }

    fn request_device(
        adapter: &wgpu::Adapter,
    ) -> impl Future<
        Output = std::result::Result<(wgpu::Device, wgpu::Queue), wgpu::RequestDeviceError>,
    > + Send {
        adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits {
                        max_bind_groups: 8,
                        ..Default::default()
                    }
                },
            },
            None,
        )
    }

    fn get_compatible_surface_format(
        surface_capabilities: &wgpu::SurfaceCapabilities,
    ) -> wgpu::TextureFormat {
        surface_capabilities
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .unwrap_or(surface_capabilities.formats[0])
    }

    /// # Errors
    /// An error may be returned if the render preparation fails
    pub fn prepare_render(
        &mut self,
        entity_store: &EntityStore,
        relationship_store: &RelationshipStore,
        asset_store: &mut AssetStore,
    ) -> Result<()> {
        self.pipeline.prepare(
            &mut self.rendering_context,
            entity_store,
            relationship_store,
            asset_store,
        )
    }

    /// # Errors
    /// An error may be returned if the rendering fails
    pub fn render(
        &mut self,
        #[cfg(feature = "egui")] egui_context: egui::Context,
        #[cfg(feature = "egui")] egui_output: egui::FullOutput,
    ) -> Result<()> {
        // TODO add proper error handling
        let output = self
            .rendering_context
            .surface
            .get_current_texture()
            .expect("Couldn't get surface texture");
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            self.rendering_context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        self.pipeline
            .render(&mut encoder, &view, &mut self.rendering_context)?;

        #[cfg(feature = "egui")]
        let tdelta = self.begin_egui_pass(&mut encoder, &view, &egui_context, egui_output);

        self.rendering_context
            .queue
            .submit(std::iter::once(encoder.finish()));
        output.present();

        #[cfg(feature = "egui")]
        self.end_egui_pass(tdelta);
        self.rendering_context.draw_commands.clear();
        Ok(())
    }

    #[cfg(feature = "egui")]
    fn end_egui_pass(&mut self, tdelta: egui::TexturesDelta) {
        self.egui_pass.remove_textures(tdelta).unwrap();
    }

    #[cfg(feature = "egui")]
    fn begin_egui_pass(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        egui_context: &egui::Context,
        egui_output: egui::FullOutput,
    ) -> egui::TexturesDelta {
        #[allow(clippy::cast_possible_truncation)]
        let screen_descriptor = egui_wgpu_backend::ScreenDescriptor {
            physical_width: self.window.inner_size().width,
            physical_height: self.window.inner_size().height,
            scale_factor: self.window.scale_factor() as f32,
        };
        let paint_jobs = egui_context.tessellate(egui_output.shapes);
        let tdelta = egui_output.textures_delta;
        self.egui_pass
            .add_textures(
                &self.rendering_context.device,
                &self.rendering_context.queue,
                &tdelta,
            )
            .unwrap();
        self.egui_pass.update_buffers(
            &self.rendering_context.device,
            &self.rendering_context.queue,
            &paint_jobs,
            &screen_descriptor,
        );
        self.egui_pass
            .execute(encoder, view, &paint_jobs, &screen_descriptor, None)
            .unwrap();
        tdelta
    }

    pub fn resize(&mut self, new_size: WindowSize) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }

        self.rendering_context.size =
            winit::dpi::PhysicalSize::new(new_size.width, new_size.height);
        self.rendering_context.surface_configuration.width = new_size.width;
        self.rendering_context.surface_configuration.height = new_size.height;
        self.rendering_context.surface.configure(
            &self.rendering_context.device,
            &self.rendering_context.surface_configuration,
        );
        self.rendering_context
            .texture_cache
            .on_window_resize(&self.rendering_context.device, new_size);
    }
}

pub struct DrawCommand {
    pub vertex_buffer: usize,
    pub index_buffer: Option<usize>,
    pub element_count: u32,
    pub vertex_count: u32,
    pub material_handle: AssetHandle<MaterialAsset>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshUniform {
    pub world_transform: [[f32; 4]; 4],
    pub inverse_world_transform: [[f32; 4]; 4],
    _padding: [u64; 16],
}

pub struct RenderingContext {
    pub size: PhysicalSize<u32>,
    pub surface: wgpu::Surface,
    pub surface_configuration: wgpu::SurfaceConfiguration,
    pub pipelines: HashMap<String, wgpu::RenderPipeline>,
    pub shader_modules: HashMap<String, wgpu::ShaderModule>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub draw_commands: Vec<DrawCommand>,
    pub vertex_buffers: Vec<wgpu::Buffer>,
    pub index_buffers: Vec<wgpu::Buffer>,
    pub texture_cache: TextureCache,
    pub material_cache: MaterialCache,
    pub model_cache: ModelCache,
}
