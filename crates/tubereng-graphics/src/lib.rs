#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use material::{MaterialAsset, MaterialCache};
use std::{collections::HashMap, future::Future};
use texture::TextureCache;
use tubereng_assets::{AssetHandle, AssetStore};

use camera::{ActiveCamera, Camera, CameraUniform, OPENGL_TO_WGPU_MATRIX};
use geometry::Model;
use render_graph::{RenderGraph, RenderPass};
use tubereng_core::Transform;
use tubereng_ecs::{entity::EntityStore, query::Q};

use wgpu::{util::DeviceExt, BindGroupLayoutDescriptor, BindGroupLayoutEntry};
use winit::{dpi::PhysicalSize, window::Window};

#[derive(Debug)]
pub struct Cube {
    pub material: AssetHandle<MaterialAsset>,
}

pub mod camera;
pub mod geometry;
pub mod material;
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

pub struct Renderer {
    _window: Window,
    size: PhysicalSize<u32>,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_configuration: wgpu::SurfaceConfiguration,

    pipelines: HashMap<String, wgpu::RenderPipeline>,
    shader_modules: HashMap<String, wgpu::ShaderModule>,
    models: HashMap<String, Model>,
    vertex_buffers: Vec<wgpu::Buffer>,
    index_buffers: Vec<wgpu::Buffer>,
    draw_commands: Vec<DrawCommand>,
    texture_cache: TextureCache,
    material_cache: MaterialCache,
    material_bind_group_layout: wgpu::BindGroupLayout,
    mesh_uniform_buffer: wgpu::Buffer,
    mesh_bind_group_layout: wgpu::BindGroupLayout,
    mesh_bind_group: wgpu::BindGroup,
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

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let mut shader_modules = HashMap::new();
        shader_modules.insert("shader".into(), shader);

        let camera_uniform = CameraUniform::new();
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let (camera_bind_group_layout, camera_bind_group) =
            Self::create_camera_bind_group(&device, &camera_buffer);

        let mesh_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("mesh_uniform_buffer"),
            size: (std::mem::size_of::<MeshUniform>() * 100) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });
        let (mesh_bind_group_layout, mesh_bind_group) =
            Self::create_mesh_bind_group(&device, &mesh_uniform_buffer);
        let material_bind_group_layout = Self::create_material_bind_group_layout(&device);
        let mut vertex_buffers = vec![];
        let mut index_buffers = vec![];
        let mut models = HashMap::new();
        models.insert(
            "_cube".into(),
            Model::new_cube(&device, &mut vertex_buffers, &mut index_buffers),
        );

        let material_cache = MaterialCache::new(&device);

        Self {
            _window: window,
            size,
            surface,
            device,
            queue,
            surface_configuration,
            pipelines: HashMap::new(),
            shader_modules,
            camera_uniform,
            camera_buffer,
            camera_bind_group_layout,
            camera_bind_group,
            models,
            draw_commands: vec![],
            vertex_buffers,
            index_buffers,
            mesh_uniform_buffer,
            mesh_bind_group_layout,
            mesh_bind_group,
            texture_cache: TextureCache::new(),
            material_cache,
            material_bind_group_layout,
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
    ) -> impl Future<Output = Result<(wgpu::Device, wgpu::Queue), wgpu::RequestDeviceError>> + Send
    {
        adapter.request_device(
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

    fn create_mesh_bind_group(
        device: &wgpu::Device,
        mesh_uniform_buffer: &wgpu::Buffer,
    ) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
        let mesh_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("mesh_bind_group_layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: wgpu::BufferSize::new(
                        std::mem::size_of::<MeshUniform>() as wgpu::BufferAddress
                    ),
                },
                count: None,
            }],
        });

        let mesh_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("mesh_bind_group"),
            layout: &mesh_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: mesh_uniform_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(
                        std::mem::size_of::<MeshUniform>() as wgpu::BufferAddress
                    ),
                }),
            }],
        });

        (mesh_bind_group_layout, mesh_bind_group)
    }

    fn create_camera_bind_group(
        device: &wgpu::Device,
        camera_buffer: &wgpu::Buffer,
    ) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
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
        (camera_bind_group_layout, camera_bind_group)
    }

    pub fn prepare_render(&mut self, entity_store: &EntityStore, asset_store: &mut AssetStore) {
        let camera_query = Q::<(&ActiveCamera, &Camera, &Transform)>::new(entity_store);
        let (_, camera, camera_transform) = camera_query.iter().next().expect("Camera not found");
        self.camera_uniform.set_view_projection_matrix(
            OPENGL_TO_WGPU_MATRIX
                * *camera.projection_matrix()
                * camera_transform
                    .as_matrix4()
                    .try_inverse()
                    .expect("No inverse for camera transform matrix"),
        );
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );

        let cube_model = &self.models["_cube"];
        for (i, (cube, transform)) in Q::<(&Cube, &Transform)>::new(entity_store)
            .iter()
            .enumerate()
        {
            let cube_material_handle = cube.material;
            if !self.material_cache.has(cube_material_handle) {
                self.material_cache.load(
                    cube_material_handle,
                    asset_store,
                    &mut self.texture_cache,
                    &self.material_bind_group_layout,
                    &self.device,
                    &self.queue,
                );
            }

            for mesh in &cube_model.meshes {
                self.draw_commands.push(DrawCommand {
                    vertex_buffer: mesh.vertex_buffer,
                    index_buffer: mesh.index_buffer,
                    element_count: mesh.element_count,
                    material_handle: cube_material_handle,
                });

                self.queue.write_buffer(
                    &self.mesh_uniform_buffer,
                    (i * std::mem::size_of::<MeshUniform>()) as u64,
                    bytemuck::cast_slice(&[MeshUniform {
                        world_transform: transform.as_matrix4().into(),
                        _padding: [0; 24],
                    }]),
                );
            }
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
        RenderPass::new("render_pass", &mut render_graph)
            .with_shader("shader")
            .with_render_target(render_target)
            .dispatch(move |rpass, draw_command, material_cache| {
                let material = material_cache.get(draw_command.material_handle);
                material.bind(2, rpass);
                if draw_command.index_buffer.is_some() {
                    rpass.draw_indexed(0..draw_command.element_count, 0, 0..1);
                } else {
                    rpass.draw(0..draw_command.element_count, 0..1);
                }
            });

        render_graph.execute(&mut RenderingContext::new(&mut encoder, self));

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        self.draw_commands.clear();
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

    fn create_material_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
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
        })
    }
}

pub struct DrawCommand {
    vertex_buffer: usize,
    index_buffer: Option<usize>,
    element_count: u32,
    material_handle: AssetHandle<MaterialAsset>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshUniform {
    world_transform: [[f32; 4]; 4],
    _padding: [u64; 24],
}

pub struct RenderingContext<'a> {
    command_encoder: &'a mut wgpu::CommandEncoder,
    pipelines: &'a mut HashMap<String, wgpu::RenderPipeline>,
    draw_commands: &'a Vec<DrawCommand>,
    device: &'a wgpu::Device,
    vertex_buffers: &'a Vec<wgpu::Buffer>,
    index_buffers: &'a Vec<wgpu::Buffer>,
    surface_configuration: &'a wgpu::SurfaceConfiguration,
    material_cache: &'a MaterialCache,
    shader_modules: &'a HashMap<String, wgpu::ShaderModule>,
    camera_bind_group_layout: &'a wgpu::BindGroupLayout,
    camera_bind_group: &'a wgpu::BindGroup,
    mesh_bind_group_layout: &'a wgpu::BindGroupLayout,
    mesh_bind_group: &'a wgpu::BindGroup,
    material_bind_group_layout: &'a wgpu::BindGroupLayout,
}

impl<'a> RenderingContext<'a> {
    pub fn new(encoder: &'a mut wgpu::CommandEncoder, renderer: &'a mut Renderer) -> Self {
        Self {
            command_encoder: encoder,
            pipelines: &mut renderer.pipelines,
            draw_commands: &renderer.draw_commands,
            device: &renderer.device,
            vertex_buffers: &renderer.vertex_buffers,
            index_buffers: &renderer.index_buffers,
            surface_configuration: &renderer.surface_configuration,
            shader_modules: &renderer.shader_modules,
            camera_bind_group_layout: &renderer.camera_bind_group_layout,
            camera_bind_group: &renderer.camera_bind_group,
            mesh_bind_group_layout: &renderer.mesh_bind_group_layout,
            mesh_bind_group: &renderer.mesh_bind_group,
            material_bind_group_layout: &renderer.material_bind_group_layout,
            material_cache: &renderer.material_cache,
        }
    }
}
