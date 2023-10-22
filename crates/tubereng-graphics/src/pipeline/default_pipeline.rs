use crate::{
    camera::{ActiveCamera, Camera, OPENGL_TO_WGPU_MATRIX},
    color::srgb_perceived_lightness,
    geometry::MeshAsset,
    material::MaterialAsset,
    render_graph::{RenderGraph, RenderPass},
    texture::{DepthBufferTextureHandle, TextureCache},
    DrawCommand, Result,
};
use std::collections::HashMap;

use tubereng_assets::{AssetHandle, AssetStore};
use tubereng_core::Transform;
use tubereng_ecs::{
    entity::EntityStore,
    query::Q,
    relationship::{ChildOf, RelationshipStore},
};
use tubereng_math::matrix::{Identity, Matrix4f};
use wgpu::util::{BufferInitDescriptor, DeviceExt};

use crate::{camera::CameraUniform, MeshUniform, RenderingContext};

use super::RenderPipeline;

const SKY_TOP_COLOR: [f32; 4] = [0.192, 0.302, 0.475, 1.0];
const SKY_BOTTOM_COLOR: [f32; 4] = [0.324, 0.179, 0.069, 1.0];
pub struct DefaultRenderPipeline {
    render_debug_grid: bool,
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

    light_storage: LightStorage,
    light_storage_buffer: wgpu::Buffer,
    light_storage_bind_group_layout: wgpu::BindGroupLayout,
    light_storage_bind_group: wgpu::BindGroup,
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

    fn create_light_storage_bind_group(
        device: &wgpu::Device,
        light_storage_buffer: &wgpu::Buffer,
    ) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
        let light_storage_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("light_storage_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let light_storage_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("light_storage_bind_group"),
            layout: &light_storage_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_storage_buffer.as_entire_binding(),
            }],
        });
        (light_storage_bind_group_layout, light_storage_bind_group)
    }

    fn add_skybox_pass<'a>(
        &'a self,
        render_graph: &mut RenderGraph<'a>,
        render_target: crate::render_graph::RenderTargetId,
    ) {
        RenderPass::new("skybox", render_graph)
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
    }

    fn extract_lights(
        &mut self,
        entity_store: &EntityStore,
        relationship_store: &RelationshipStore,
    ) {
        self.light_storage.point_light_count = 0;
        for (i, (point_light, transform)) in
            Q::<(&crate::light::PointLight, &Transform)>::new(entity_store, relationship_store)
                .iter()
                .take(10)
                .enumerate()
        {
            self.light_storage.point_light_count += 1;
            self.light_storage.point_lights[i] = PointLight {
                position: [
                    transform.translation.x,
                    transform.translation.y,
                    transform.translation.z,
                ],
                _padding: 0,
                color: [
                    point_light.color.x,
                    point_light.color.y,
                    point_light.color.z,
                ],
                _padding2: 0,
                constant: point_light.constant,
                linear: point_light.linear,
                quadratic: point_light.quadratic,
                _padding3: 0,
            };
        }
    }
}

#[derive(Default)]
pub struct DefaultRenderPipelineSettings {
    pub render_debug_grid: bool,
}

impl RenderPipeline for DefaultRenderPipeline {
    type RenderPipelineSettings = DefaultRenderPipelineSettings;
    fn new(
        render_pipeline_settings: &Self::RenderPipelineSettings,
        device: &wgpu::Device,
        surface_configuration: &wgpu::SurfaceConfiguration,
        texture_cache: &mut TextureCache,
        shader_modules: &mut HashMap<String, wgpu::ShaderModule>,
    ) -> Self {
        load_shaders(device, shader_modules);

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
            top_color: SKY_TOP_COLOR,
            bottom_color: SKY_BOTTOM_COLOR,
        };

        let gradient_uniform_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("gradient_uniform_buffer"),
                contents: bytemuck::cast_slice(&[gradient_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let (gradient_uniform_bind_group_layout, gradient_uniform_bind_group) =
            Self::create_gradient_uniform_bind_group(device, &gradient_uniform_buffer);

        let ambient_light =
            srgb_perceived_lightness(SKY_TOP_COLOR[0], SKY_TOP_COLOR[1], SKY_TOP_COLOR[2])
                + srgb_perceived_lightness(
                    SKY_BOTTOM_COLOR[0],
                    SKY_BOTTOM_COLOR[1],
                    SKY_BOTTOM_COLOR[2],
                ) / 2.0;

        let light_storage = LightStorage {
            ambient_light_factor: ambient_light,
            point_light_count: 0,
            point_lights: [PointLight::default(); 10],
            _padding: 0,
        };

        let light_storage_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("light_storage_buffer"),
            contents: bytemuck::cast_slice(&[light_storage]),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
        });
        let (light_storage_bind_group_layout, light_storage_bind_group) =
            Self::create_light_storage_bind_group(device, &light_storage_buffer);

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
            light_storage,
            light_storage_buffer,
            light_storage_bind_group_layout,
            light_storage_bind_group,
            render_debug_grid: render_pipeline_settings.render_debug_grid,
        }
    }

    #[allow(clippy::too_many_lines)]
    fn prepare(
        &mut self,
        ctx: &mut RenderingContext,
        entity_store: &EntityStore,
        relationship_store: &RelationshipStore,
        asset_store: &mut AssetStore,
    ) -> Result<()> {
        let camera_query =
            Q::<(&ActiveCamera, &Camera, &Transform)>::new(entity_store, relationship_store);
        let (_, camera, camera_transform) = camera_query.iter().next().expect("Camera not found");
        let camera_view_projection_matrix = OPENGL_TO_WGPU_MATRIX
            * *camera.projection_matrix()
            * camera_transform
                .as_matrix4()
                .try_inverse()
                .expect("No inverse for camera transform matrix");
        self.camera_uniform
            .set_view_projection_matrix(camera_view_projection_matrix);
        self.camera_uniform
            .set_position(camera_transform.translation);
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

        let mut entities_to_process = entity_store
            .entity_ids()
            .difference(&relationship_store.all_sources_of::<ChildOf>())
            .map(|i| (None, *i))
            .collect::<Vec<_>>();

        let mut model_uniforms = vec![];
        let mut transforms: Vec<Matrix4f> = vec![];

        let mut i = 0;
        while let Some((parent, entity)) = entities_to_process.pop() {
            let Some((mesh, material, transform)) = Q::<(
                &AssetHandle<MeshAsset>,
                &AssetHandle<MaterialAsset>,
                &Transform,
            )>::new(entity_store, relationship_store)
            .with_id(entity) else {
                continue;
            };

            entities_to_process.extend_from_slice(
                &relationship_store
                    .sources_of::<ChildOf>(entity)
                    .iter()
                    .map(|j| (Some(i), *j))
                    .collect::<Vec<_>>(),
            );

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

            let mesh_handle = *mesh;
            if !ctx.model_cache.has(mesh_handle) {
                ctx.model_cache.load(
                    mesh_handle,
                    asset_store,
                    &mut ctx.vertex_buffers,
                    &mut ctx.index_buffers,
                    &ctx.device,
                )?;
            }

            let mut transform_matrix = transform.as_matrix4();
            if let Some(parent) = parent {
                transform_matrix *= transforms[parent];
            }

            let model = ctx
                .model_cache
                .get(mesh_handle)
                .expect("Model not found in cache");
            for mesh in &model.meshes {
                ctx.draw_commands.push(DrawCommand {
                    vertex_buffer: mesh.vertex_buffer,
                    index_buffer: mesh.index_buffer,
                    element_count: mesh.element_count,
                    vertex_count: mesh.vertex_count,
                    material_handle,
                });
                model_uniforms.push(MeshUniform {
                    world_transform: transform_matrix.into(),
                    inverse_world_transform: transform_matrix.try_inverse().unwrap().into(),
                    _padding: [0u64; 16],
                });
                transforms.push(transform_matrix);
            }
            i += 1;
        }
        ctx.queue.write_buffer(
            &self.mesh_uniform_buffer,
            0,
            bytemuck::cast_slice(&model_uniforms),
        );

        // for (i, (mesh, material, transform)) in Q::<(
        //     &AssetHandle<MeshAsset>,
        //     &AssetHandle<MaterialAsset>,
        //     &Transform,
        // )>::new(entity_store, relationship_store)
        // .iter()
        // .enumerate()
        // {
        //     let material_handle = *material;
        //     if !ctx.material_cache.has(material_handle) {
        //         ctx.material_cache.load(
        //             material_handle,
        //             asset_store,
        //             &mut ctx.texture_cache,
        //             &self.material_bind_group_layout,
        //             &ctx.device,
        //             &ctx.queue,
        //         )?;
        //     }

        //     let mesh_handle = *mesh;
        //     if !ctx.model_cache.has(mesh_handle) {
        //         ctx.model_cache.load(
        //             mesh_handle,
        //             asset_store,
        //             &mut ctx.vertex_buffers,
        //             &mut ctx.index_buffers,
        //             &ctx.device,
        //         )?;
        //     }

        //     let model = ctx
        //         .model_cache
        //         .get(mesh_handle)
        //         .expect("Model not found in cache");
        //     for mesh in &model.meshes {
        //         ctx.draw_commands.push(DrawCommand {
        //             vertex_buffer: mesh.vertex_buffer,
        //             index_buffer: mesh.index_buffer,
        //             element_count: mesh.element_count,
        //             vertex_count: mesh.vertex_count,
        //             material_handle,
        //         });

        //         ctx.queue.write_buffer(
        //             &self.mesh_uniform_buffer,
        //             (i * std::mem::size_of::<MeshUniform>()) as u64,
        //             bytemuck::cast_slice(&[MeshUniform {
        //                 world_transform: transform.as_matrix4().into(),
        //                 inverse_world_transform: transform
        //                     .as_matrix4()
        //                     .try_inverse()
        //                     .unwrap()
        //                     .into(),
        //                 _padding: [0u64; 16],
        //             }]),
        //         );
        //     }
        // }

        self.extract_lights(entity_store, relationship_store);

        ctx.queue.write_buffer(
            &self.light_storage_buffer,
            0,
            bytemuck::cast_slice(&[self.light_storage]),
        );

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

        self.add_skybox_pass(&mut render_graph, render_target);

        RenderPass::new("render_pass", &mut render_graph)
            .with_shader("shader")
            .with_depth_buffer(self.depth_buffer_texture_handle, true)
            .with_render_target(render_target)
            .with_bind_group(&self.camera_bind_group_layout, &self.camera_bind_group)
            .with_bind_group(&self.mesh_bind_group_layout, &self.mesh_bind_group)
            .with_bind_group_layout(&self.material_bind_group_layout)
            .with_bind_group(
                &self.light_storage_bind_group_layout,
                &self.light_storage_bind_group,
            )
            .with_bind_group(
                &self.light_storage_bind_group_layout,
                &self.light_storage_bind_group,
            )
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
                        rpass.set_bind_group(3, bind_groups[2], &[]);
                        rpass.set_bind_group(4, bind_groups[3], &[]);
                        let material = material_cache
                            .get(draw_command.material_handle)
                            .expect("Material not found in cache");
                        material.bind(2, rpass);
                        if draw_command.index_buffer.is_some() {
                            rpass.draw_indexed(0..draw_command.element_count, 0, 0..1);
                        } else {
                            rpass.draw(0..draw_command.vertex_count, 0..1);
                        }
                    }
                },
            );

        if self.render_debug_grid {
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
        }

        render_graph.execute(command_encoder, ctx);

        Ok(())
    }
}

fn load_shaders(device: &wgpu::Device, shader_modules: &mut HashMap<String, wgpu::ShaderModule>) {
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

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct LightStorage {
    ambient_light_factor: f32,
    point_light_count: u32,
    _padding: u64,
    point_lights: [PointLight; 10],
}

#[repr(C)]
#[derive(Default, Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct PointLight {
    position: [f32; 3],
    _padding: u32,
    color: [f32; 3],
    _padding2: u32,
    constant: f32,
    linear: f32,
    quadratic: f32,
    _padding3: u32,
}
