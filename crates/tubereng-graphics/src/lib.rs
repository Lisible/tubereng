#![warn(clippy::pedantic)]

use std::collections::HashMap;

use render_graph::{RenderGraph, RenderPass};
use winit::{dpi::PhysicalSize, window::Window};

pub mod render_graph;

#[derive(Clone, Copy)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

impl WindowSize {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

pub struct Renderer {
    _window: Window,
    size: PhysicalSize<u32>,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_configuration: wgpu::SurfaceConfiguration,

    pipelines: HashMap<String, wgpu::RenderPipeline>,
    shader_modules: HashMap<String, wgpu::ShaderModule>,
}

impl Renderer {
    pub async fn new(window: Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Safety: WGPURenderer owns both the window and the surface, so the window will live as
        // long as the surface
        let surface = unsafe { instance.create_surface(&window) }.expect("Surface creation failed");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("No video adapter found");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                },
                None,
            )
            .await
            .expect("Couldn't request device");

        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .unwrap_or(surface_capabilities.formats[0]);
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

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let mut shader_modules = HashMap::new();
        shader_modules.insert("shader".into(), shader);

        Self {
            _window: window,
            size,
            surface,
            device,
            queue,
            surface_configuration,
            pipelines: HashMap::new(),
            shader_modules,
        }
    }

    pub fn render(&mut self) {
        // TODO add proper error handling
        let output = self.surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let mut render_graph = RenderGraph::new();
        let render_target = render_graph.register_render_target(view);
        RenderPass::new("render_pass", &mut render_graph)
            .with_shader("shader")
            .with_render_target(render_target)
            .dispatch(|rpass| {
                rpass.draw(0..3, 0..1);
            });

        RenderPass::new("render_pass2", &mut render_graph)
            .with_shader("shader")
            .with_render_target(render_target)
            .dispatch(|rpass| {
                rpass.draw(0..3, 0..1);
            });

        render_graph.execute(
            &mut encoder,
            &self.device,
            &self.shader_modules,
            &mut self.pipelines,
        );

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    pub fn resize(&mut self, new_size: WindowSize) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }

        self.size = winit::dpi::PhysicalSize::new(new_size.width, new_size.height);
        self.surface_configuration.width = new_size.width;
        self.surface_configuration.height = new_size.height;
        self.surface
            .configure(&self.device, &self.surface_configuration);
    }
}
