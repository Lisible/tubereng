use crate::{
    camera::{ActiveCamera, Camera, OPENGL_TO_WGPU_MATRIX},
    geometry::ModelAsset,
    material::MaterialAsset,
    render_graph::{RenderGraph, RenderPass},
    texture::{DepthBufferTextureHandle, TextureCache},
    DrawCommand, Result,
};
use std::collections::HashMap;

use tubereng_assets::{AssetHandle, AssetStore};
use tubereng_core::Transform;
use tubereng_ecs::{entity::EntityStore, query::Q};
use tubereng_math::matrix::{Identity, Matrix4f};
use wgpu::util::DeviceExt;

use crate::{camera::CameraUniform, MeshUniform, RenderingContext};

use super::RenderPipeline;

pub struct DefaultRenderPipeline {
    material_bind_group_layout: wgpu::BindGroupLayout,
    mesh_uniform_buffer: wgpu::Buffer,
    mesh_bind_group_layout: wgpu::BindGroupLayout,
    mesh_bind_group: wgpu::BindGroup,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group_layout: wgpu::BindGroupLayout,
    camera_bind_group: wgpu::BindGroup,
    inverse_camera_uniform: InverseCameraUniform,
    inverse_camera_buffer: wgpu::Buffer,
    inverse_camera_bind_group_layout: wgpu::BindGroupLayout,
    inverse_camera_bind_group: wgpu::BindGroup,

    gradient_uniform_bind_group_layout: wgpu::BindGroupLayout,
    gradient_uniform_bind_group: wgpu::BindGroup,
    depth_buffer_texture_handle: DepthBufferTextureHandle,
}

impl DefaultRenderPipeline {
    fn create_mesh_bind_group(
        device: &wgpu::Device,
        mesh_uniform_buffer: &wgpu::Buffer,
    ) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
        let mesh_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("mesh_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
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
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
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

    fn create_inverse_camera_bind_group(
        device: &wgpu::Device,
        inverse_camera_buffer: &wgpu::Buffer,
    ) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
        let inverse_camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("inverse_camera_bind_group_layout"),
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
        let inverse_camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("inverse_camera_bind_group"),
            layout: &inverse_camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: inverse_camera_buffer.as_entire_binding(),
            }],
        });
        (inverse_camera_bind_group_layout, inverse_camera_bind_group)
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

    fn create_gradient_uniform_bind_group(
        device: &wgpu::Device,
        gradient_uniform_buffer: &wgpu::Buffer,
    ) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
        let gradient_uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("gradient_uniform_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let gradient_uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("gradient_uniform_bind_group"),
            layout: &gradient_uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: gradient_uniform_buffer.as_entire_binding(),
            }],
        });
        (
            gradient_uniform_bind_group_layout,
            gradient_uniform_bind_group,
        )
    }
}

impl RenderPipeline for DefaultRenderPipeline {
    fn new(
        device: &wgpu::Device,
        surface_configuration: &wgpu::SurfaceConfiguration,
        texture_cache: &mut TextureCache,
        shader_modules: &mut HashMap<String, wgpu::ShaderModule>,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        shader_modules.insert("shader".into(), shader);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Gradient sky shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("gradient_sky.wgsl").into()),
        });
        shader_modules.insert("gradient_sky".into(), shader);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Debug grid shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("debug_grid.wgsl").into()),
        });
        shader_modules.insert("debug_grid".into(), shader);

        let camera_uniform = CameraUniform::new();
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let (camera_bind_group_layout, camera_bind_group) =
            Self::create_camera_bind_group(device, &camera_buffer);

        let inverse_camera_uniform = InverseCameraUniform {
            view_projection_inverse: Matrix4f::identity().into(),
        };
        let inverse_camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("view_projection_inverse"),
            contents: bytemuck::cast_slice(&[inverse_camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let (inverse_camera_bind_group_layout, inverse_camera_bind_group) =
            Self::create_inverse_camera_bind_group(device, &inverse_camera_buffer);

        let mesh_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("mesh_uniform_buffer"),
            size: (std::mem::size_of::<MeshUniform>() * 100) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });
        let (mesh_bind_group_layout, mesh_bind_group) =
            Self::create_mesh_bind_group(device, &mesh_uniform_buffer);
        let material_bind_group_layout = Self::create_material_bind_group_layout(device);

        let gradient_uniform = GradientUniform {
            top_color: [0.192, 0.302, 0.475, 1.0],
            bottom_color: [0.324, 0.179, 0.069, 1.0],
        };

        let gradient_uniform_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("gradient_uniform_buffer"),
                contents: bytemuck::cast_slice(&[gradient_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let (gradient_uniform_bind_group_layout, gradient_uniform_bind_group) =
            Self::create_gradient_uniform_bind_group(device, &gradient_uniform_buffer);

        let depth_texture_handle = texture_cache.create_depth_texture(
            device,
            "depth_buffer",
            surface_configuration.width,
            surface_configuration.height,
            true,
        );

        Self {
            material_bind_group_layout,
            mesh_uniform_buffer,
            mesh_bind_group_layout,
            mesh_bind_group,
            camera_uniform,
            camera_buffer,
            camera_bind_group_layout,
            camera_bind_group,
            gradient_uniform_bind_group_layout,
            gradient_uniform_bind_group,
            depth_buffer_texture_handle: depth_texture_handle,
            inverse_camera_uniform,
            inverse_camera_buffer,
            inverse_camera_bind_group_layout,
            inverse_camera_bind_group,
        }
    }

    fn prepare(
        &mut self,
        ctx: &mut RenderingContext,
        entity_store: &EntityStore,
        asset_store: &mut AssetStore,
    ) -> Result<()> {
        let camera_query = Q::<(&ActiveCamera, &Camera, &Transform)>::new(entity_store);
        let (_, camera, camera_transform) = camera_query.iter().next().expect("Camera not found");
        let camera_view_projection_matrix = OPENGL_TO_WGPU_MATRIX
            * *camera.projection_matrix()
            * camera_transform
                .as_matrix4()
                .try_inverse()
                .expect("No inverse for camera transform matrix");
        self.camera_uniform
            .set_view_projection_matrix(camera_view_projection_matrix);
        ctx.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );

        self.inverse_camera_uniform.view_projection_inverse =
            camera_view_projection_matrix.try_inverse().unwrap().into();
        ctx.queue.write_buffer(
            &self.inverse_camera_buffer,
            0,
            bytemuck::cast_slice(&[self.inverse_camera_uniform]),
        );

        for (i, (model, material, transform)) in Q::<(
            &AssetHandle<ModelAsset>,
            &AssetHandle<MaterialAsset>,
            &Transform,
        )>::new(entity_store)
        .iter()
        .enumerate()
        {
            let material_handle = *material;
            if !ctx.material_cache.has(material_handle) {
                ctx.material_cache.load(
                    material_handle,
                    asset_store,
                    &mut ctx.texture_cache,
                    &self.material_bind_group_layout,
                    &ctx.device,
                    &ctx.queue,
                )?;
            }

            let model_handle = *model;
            if !ctx.model_cache.has(model_handle) {
                ctx.model_cache.load(
                    model_handle,
                    asset_store,
                    &mut ctx.vertex_buffers,
                    &ctx.index_buffers,
                    &ctx.device,
                )?;
            }

            let model = ctx
                .model_cache
                .get(model_handle)
                .expect("Model not found in cache");
            for mesh in &model.meshes {
                ctx.draw_commands.push(DrawCommand {
                    vertex_buffer: mesh.vertex_buffer,
                    index_buffer: mesh.index_buffer,
                    element_count: mesh.element_count,
                    material_handle,
                });

                ctx.queue.write_buffer(
                    &self.mesh_uniform_buffer,
                    (i * std::mem::size_of::<MeshUniform>()) as u64,
                    bytemuck::cast_slice(&[MeshUniform {
                        world_transform: transform.as_matrix4().into(),
                        _padding: [0; 24],
                    }]),
                );
            }
        }

        Ok(())
    }

    fn render(
        &mut self,
        command_encoder: &mut wgpu::CommandEncoder,
        view: wgpu::TextureView,
        ctx: &mut RenderingContext,
    ) -> Result<()> {
        let mut render_graph = RenderGraph::new();
        let render_target = render_graph.register_render_target(view);

        RenderPass::new("skybox", &mut render_graph)
            .with_no_vertex_buffer()
            .with_shader("gradient_sky")
            .with_render_target(render_target)
            .with_bind_group(
                &self.gradient_uniform_bind_group_layout,
                &self.gradient_uniform_bind_group,
            )
            .dispatch(
                |rpass,
                 bind_groups,
                 _draw_commands,
                 _material_cache,
                 _vertex_buffers,
                 _index_buffers| {
                    rpass.set_bind_group(0, bind_groups[0], &[]);
                    rpass.draw(0..3, 0..1);
                },
            );

        RenderPass::new("render_pass", &mut render_graph)
            .with_shader("shader")
            .with_depth_buffer(self.depth_buffer_texture_handle, true)
            .with_render_target(render_target)
            .with_bind_group(&self.camera_bind_group_layout, &self.camera_bind_group)
            .with_bind_group(&self.mesh_bind_group_layout, &self.mesh_bind_group)
            .with_bind_group_layout(&self.material_bind_group_layout)
            .dispatch(
                |rpass,
                 bind_groups,
                 vertex_buffers,
                 index_buffers,
                 draw_commands,
                 material_cache| {
                    for (draw_command_index, draw_command) in draw_commands.iter().enumerate() {
                        rpass.set_vertex_buffer(
                            0,
                            vertex_buffers[draw_command.vertex_buffer].slice(..),
                        );
                        if let Some(index_buffer) = draw_command.index_buffer {
                            rpass.set_index_buffer(
                                index_buffers[index_buffer].slice(..),
                                wgpu::IndexFormat::Uint16,
                            );
                        }

                        rpass.set_bind_group(0, bind_groups[0], &[]);
                        rpass.set_bind_group(
                            1,
                            bind_groups[1],
                            &[u32::try_from(draw_command_index * 256).unwrap()],
                        );
                        let material = material_cache
                            .get(draw_command.material_handle)
                            .expect("Material not found in cache");
                        material.bind(2, rpass);
                        if draw_command.index_buffer.is_some() {
                            rpass.draw_indexed(0..draw_command.element_count, 0, 0..1);
                        } else {
                            rpass.draw(0..draw_command.element_count, 0..1);
                        }
                    }
                },
            );

        RenderPass::new("debug_grid", &mut render_graph)
            .with_no_vertex_buffer()
            .with_depth_buffer(self.depth_buffer_texture_handle, false)
            .with_blend_state(wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent::default(),
            })
            .with_shader("debug_grid")
            .with_render_target(render_target)
            .with_bind_group(&self.camera_bind_group_layout, &self.camera_bind_group)
            .with_bind_group(
                &self.inverse_camera_bind_group_layout,
                &self.inverse_camera_bind_group,
            )
            .dispatch(
                |rpass,
                 bind_groups,
                 _vertex_buffers,
                 _index_buffers,
                 _draw_commands,
                 _material_cache| {
                    rpass.set_bind_group(0, bind_groups[0], &[]);
                    rpass.set_bind_group(1, bind_groups[1], &[]);
                    rpass.draw(0..6, 0..1);
                },
            );

        render_graph.execute(command_encoder, ctx);

        Ok(())
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GradientUniform {
    top_color: [f32; 4],
    bottom_color: [f32; 4],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct InverseCameraUniform {
    view_projection_inverse: [[f32; 4]; 4],
}
