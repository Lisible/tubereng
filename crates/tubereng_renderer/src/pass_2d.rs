use std::collections::HashMap;

use tubereng_core::TransformCache;
use tubereng_ecs::Storage;
use tubereng_math::{matrix::Matrix4f, vector::Vector3f};
use wgpu::include_wgsl;

use crate::{
    camera,
    graphics_pipeline::RenderPass,
    mesh::Vertex,
    sprite::{AnimatedSprite, Sprite},
    texture, GraphicsState, PipelineCache,
};

struct Quad2d {
    pub(crate) transform: Matrix4f,
    texture_id: texture::Id,
    texture_rect: texture::Rect,
}
struct PendingBatch {
    pub(crate) vertices: Vec<Vertex>,
    pub(crate) texture_id: texture::Id,
}

impl PendingBatch {
    pub fn new(texture_id: texture::Id) -> Self {
        Self {
            vertices: vec![],
            texture_id,
        }
    }
}

struct BatchMetadata {
    start_vertex_index: u32,
    end_vertex_index: u32,
    texture_id: texture::Id,
}

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
pub struct PassUniform {
    view_proj: [[f32; 4]; 4],
}

pub struct Pass {
    pending_batches: Vec<PendingBatch>,
    batches_metadata: Vec<BatchMetadata>,
    #[allow(clippy::struct_field_names)]
    pass_uniform_buffer: wgpu::Buffer,
    #[allow(clippy::struct_field_names)]
    pass_uniform_bind_group_layout: wgpu::BindGroupLayout,
    #[allow(clippy::struct_field_names)]
    pass_uniform_bind_group: wgpu::BindGroup,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_groups: HashMap<texture::Id, wgpu::BindGroup>,
    vertex_buffer: wgpu::Buffer,
}

impl Pass {
    const MAX_VERTICES: usize = 10_000;
    #[must_use]
    pub fn new(device: &wgpu::Device) -> Self {
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("pass_2d_vertex_buffer"),
            size: (Self::MAX_VERTICES * std::mem::size_of::<Vertex>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("texture_bind_group_layout"),
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

        let pass_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("pass_uniform"),
            size: std::mem::size_of::<PassUniform>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let pass_uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("pass_uniform_bind_group_layout"),
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

        let pass_uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("pass_uniform_bind_group"),
            layout: &pass_uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: pass_uniform_buffer.as_entire_binding(),
            }],
        });

        Self {
            pending_batches: vec![],
            batches_metadata: vec![],
            texture_bind_group_layout,
            texture_bind_groups: HashMap::new(),
            vertex_buffer,
            pass_uniform_buffer,
            pass_uniform_bind_group,
            pass_uniform_bind_group_layout,
        }
    }

    #[allow(clippy::cast_precision_loss)]
    fn queue_quad_2d(&mut self, quad: &Quad2d, texture_info: &texture::Info) {
        let local_to_world_matrix = quad.transform;

        let texture_w = texture_info.width as f32;
        let texture_h = texture_info.height as f32;
        let quad_texture_u = quad.texture_rect.x;
        let quad_texture_v = quad.texture_rect.y;
        let quad_texture_w = quad.texture_rect.width;
        let quad_texture_h = quad.texture_rect.height;

        let top_left = local_to_world_matrix
            .transform_vec3(&Vector3f::new(0.0, 0.0, 0.0))
            .into();
        let bottom_left = local_to_world_matrix
            .transform_vec3(&Vector3f::new(0.0, quad_texture_h, 0.0))
            .into();
        let bottom_right = local_to_world_matrix
            .transform_vec3(&Vector3f::new(quad_texture_w, quad_texture_h, 0.0))
            .into();
        let top_right = local_to_world_matrix
            .transform_vec3(&Vector3f::new(quad_texture_w, 0.0, 0.0))
            .into();
        let texture_id = quad.texture_id;

        let batch = match self.pending_batches.last_mut() {
            Some(batch) if batch.texture_id == texture_id => batch,
            _ => {
                self.pending_batches.push(PendingBatch::new(texture_id));
                // SAFETY: We just added a batch to the pending batch list
                unsafe { self.pending_batches.last_mut().unwrap_unchecked() }
            }
        };

        #[allow(clippy::cast_precision_loss)]
        batch.vertices.extend_from_slice(&[
            Vertex {
                position: top_left,
                texture_coordinates: [quad_texture_u / texture_w, quad_texture_v / texture_h],
            },
            Vertex {
                position: bottom_left,
                texture_coordinates: [
                    quad_texture_u / texture_w,
                    (quad_texture_v + quad_texture_h) / texture_h,
                ],
            },
            Vertex {
                position: bottom_right,
                texture_coordinates: [
                    (quad_texture_u + quad_texture_w) / texture_w,
                    (quad_texture_v + quad_texture_h) / texture_h,
                ],
            },
            Vertex {
                position: bottom_right,
                texture_coordinates: [
                    (quad_texture_u + quad_texture_w) / texture_w,
                    (quad_texture_v + quad_texture_h) / texture_h,
                ],
            },
            Vertex {
                position: top_right,
                texture_coordinates: [
                    (quad_texture_u + quad_texture_w) / texture_w,
                    quad_texture_v / texture_h,
                ],
            },
            Vertex {
                position: top_left,
                texture_coordinates: [quad_texture_u / texture_w, quad_texture_v / texture_h],
            },
        ]);
    }

    #[must_use]
    pub fn create_pass_2d_pipeline(
        device: &wgpu::Device,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
        surface_texture_format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        let shader_module = device.create_shader_module(include_wgsl!("./pass_2d.wgsl"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("pass_2d_pipeline"),
                bind_group_layouts,
                push_constant_ranges: &[],
            });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[Vertex::layout()],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_texture_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent::default(),
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        })
    }

    fn create_texture_bind_group_for_texture_if_required(
        &mut self,
        texture: texture::Id,
        gfx: &std::cell::Ref<'_, GraphicsState<'_>>,
    ) {
        if let std::collections::hash_map::Entry::Vacant(e) =
            self.texture_bind_groups.entry(texture)
        {
            let texture = gfx.texture_cache.get(texture);
            let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let texture_sampler = gfx.device().create_sampler(&wgpu::SamplerDescriptor {
                label: None,
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            });

            let texture_bind_group = gfx.device().create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&texture_sampler),
                    },
                ],
            });

            e.insert(texture_bind_group);
        }
    }
}

impl RenderPass for Pass {
    fn prepare(&mut self, storage: &Storage) {
        let gfx = storage
            .resource::<GraphicsState>()
            .expect("Graphics state should be present");

        let (camera_id, (camera, _)) = storage
            .query::<(&camera::D2, &camera::Active)>()
            .iter_with_ids()
            .next()
            .expect("An active 2d camera should be present in the scene");

        let transform_cache = storage
            .resource::<TransformCache>()
            .expect("TransformCache resource should be present");
        let camera_transform = transform_cache.get(camera_id);
        let inverse_transform = camera_transform.try_inverse().unwrap();
        gfx.queue().write_buffer(
            &self.pass_uniform_buffer,
            0,
            bytemuck::cast_slice(&[PassUniform {
                view_proj: (*camera.projection() * inverse_transform).into(),
            }]),
        );

        for (id, sprite) in storage.query::<&Sprite>().iter_with_ids() {
            self.create_texture_bind_group_for_texture_if_required(sprite.texture, &gfx);
            let texture_info = gfx.texture_cache.info(sprite.texture);
            #[allow(clippy::cast_precision_loss)]
            self.queue_quad_2d(
                &Quad2d {
                    transform: transform_cache.get(id),
                    texture_id: sprite.texture,
                    texture_rect: sprite.texture_rect.clone().unwrap_or(texture::Rect {
                        x: 0.0,
                        y: 0.0,
                        width: texture_info.width as f32,
                        height: texture_info.height as f32,
                    }),
                },
                texture_info,
            );
        }

        for (id, animated_sprite) in storage.query::<&AnimatedSprite>().iter_with_ids() {
            self.create_texture_bind_group_for_texture_if_required(
                animated_sprite.texture_atlas,
                &gfx,
            );
            let texture_info = gfx.texture_cache.info(animated_sprite.texture_atlas);
            let animation = &animated_sprite.animation;
            let rect =
                animation.animations[animation.current_animation][animation.current_frame].clone();
            #[allow(clippy::cast_precision_loss)]
            self.queue_quad_2d(
                &Quad2d {
                    transform: transform_cache.get(id),
                    texture_id: animated_sprite.texture_atlas,
                    texture_rect: rect,
                },
                texture_info,
            );
        }

        let mut vertex_count = 0u32;
        self.batches_metadata.clear();
        for batch in self.pending_batches.drain(..) {
            let start_vertex_index = vertex_count;
            gfx.wgpu_state.queue.write_buffer(
                &self.vertex_buffer,
                (vertex_count as usize * std::mem::size_of::<Vertex>()) as wgpu::BufferAddress,
                bytemuck::cast_slice(&batch.vertices),
            );
            vertex_count += u32::try_from(batch.vertices.len()).unwrap();

            let end_vertex_index = vertex_count;
            self.batches_metadata.push(BatchMetadata {
                start_vertex_index,
                end_vertex_index,
                texture_id: batch.texture_id,
            });
        }
    }

    fn execute(
        &self,
        gfx: &mut GraphicsState,
        encoder: &mut wgpu::CommandEncoder,
        surface_texture_view: &wgpu::TextureView,
        storage: &Storage,
    ) {
        let mut pipeline_cache = storage.resource_mut::<PipelineCache>().unwrap();
        if !pipeline_cache.has("pass_2d_pipeline") {
            pipeline_cache.insert(
                "pass_2d_pipeline",
                Self::create_pass_2d_pipeline(
                    gfx.device(),
                    &[
                        &self.pass_uniform_bind_group_layout,
                        &self.texture_bind_group_layout,
                    ],
                    gfx.surface_texture_format(),
                ),
            );
        }
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("pass_2d"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: surface_texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        rpass.set_pipeline(pipeline_cache.get("pass_2d_pipeline").unwrap());
        rpass.set_bind_group(0, &self.pass_uniform_bind_group, &[]);
        for batch in &self.batches_metadata {
            rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            let texture_bind_group = &self.texture_bind_groups[&batch.texture_id];
            rpass.set_bind_group(1, texture_bind_group, &[]);
            rpass.draw(batch.start_vertex_index..batch.end_vertex_index, 0..1);
        }
    }
}
