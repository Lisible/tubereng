#![warn(clippy::pedantic)]

use std::{collections::HashMap, f32::consts::PI};

use camera::{Camera, CameraUniform};
use render_graph::{RenderGraph, RenderPass};
use tubereng_core::Transform;
use tubereng_math::{quaternion::Quaternion, vector::Vector3f};
use wgpu::util::DeviceExt;
use winit::{dpi::PhysicalSize, window::Window};

#[derive(Debug)]
pub struct Cube;

pub mod camera;
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
    vertex_buffer: wgpu::Buffer,
    vertex_count: u32,
    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group_layout: wgpu::BindGroupLayout,
    camera_bind_group: wgpu::BindGroup,
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

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let vertex_count = VERTICES.len() as u32;

        let camera = Camera::new(
            (0.0, 1.0, 2.0).into(),
            (0.0, 0.0, 0.0).into(),
            (0.0, 1.0, 0.0).into(),
            800.0 / 600.0,
            45.0,
            0.1,
            100.0,
        );
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.set_view_projection_matrix(camera.projection_matrix());

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("camera_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera_bind_group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        Self {
            _window: window,
            size,
            surface,
            device,
            queue,
            surface_configuration,
            pipelines: HashMap::new(),
            shader_modules,
            vertex_buffer,
            vertex_count,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group_layout,
            camera_bind_group,
        }
    }

    pub fn render(&mut self) {
        // TODO add proper error handling
        let output = self
            .surface
            .get_current_texture()
            .expect("Couldn't get surface texture");
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
        let vertex_count = self.vertex_count;
        RenderPass::new("render_pass", &mut render_graph)
            .with_shader("shader")
            .with_render_target(render_target)
            .dispatch(move |rpass| {
                rpass.draw(0..vertex_count, 0..1);
            });

        RenderPass::new("render_pass2", &mut render_graph)
            .with_shader("shader")
            .with_render_target(render_target)
            .dispatch(move |rpass| {
                rpass.draw(0..vertex_count, 0..1);
            });

        render_graph.execute(
            &mut encoder,
            &self.device,
            &self.surface_configuration,
            &self.shader_modules,
            &mut self.pipelines,
            &self.vertex_buffer,
            &self.camera_bind_group_layout,
            &self.camera_bind_group,
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

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

    fn buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.5, 0.0],
        color: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.0],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.0],
        color: [0.0, 0.0, 1.0],
    },
];
