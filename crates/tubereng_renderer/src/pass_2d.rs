use tubereng_core::Transform;
use tubereng_ecs::Storage;
use tubereng_math::vector::Vector3f;
use wgpu::include_wgsl;

use crate::{
    material, mesh::Vertex, render_graph::RenderPass, sprite::Sprite, GraphicsState,
    RenderPipelines,
};

pub(crate) struct Quad2d {
    pub(crate) transform: Transform,
    pub(crate) material: material::Id,
}
struct PendingBatch {
    pub(crate) vertices: Vec<Vertex>,
    pub(crate) material_id: material::Id,
}

impl PendingBatch {
    pub fn new(material_id: material::Id) -> Self {
        Self {
            vertices: vec![],
            material_id,
        }
    }
}

struct BatchMetadata {
    start_vertex_index: u32,
    end_vertex_index: u32,
    material_id: material::Id,
}

pub struct Pass {
    pending_batches: Vec<PendingBatch>,
    batches_metadata: Vec<BatchMetadata>,
    vertex_buffer: wgpu::Buffer,
}

impl Pass {
    const MAX_VERTICES: usize = 10_000;
    pub fn new(device: &wgpu::Device) -> Self {
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("pass_2d_vertex_buffer"),
            size: (Self::MAX_VERTICES * std::mem::size_of::<Vertex>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pending_batches: vec![],
            batches_metadata: vec![],
            vertex_buffer,
        }
    }

    pub fn queue_quad_2d(&mut self, quad: &Quad2d) {
        let local_to_world_matrix = quad.transform.as_matrix4();
        let top_left = local_to_world_matrix
            .transform_vec3(&Vector3f::new(0.0, 0.0, 0.0))
            .into();
        let bottom_left = local_to_world_matrix
            .transform_vec3(&Vector3f::new(0.0, 1.0, 0.0))
            .into();
        let bottom_right = local_to_world_matrix
            .transform_vec3(&Vector3f::new(1.0, 1.0, 0.0))
            .into();
        let top_right = local_to_world_matrix
            .transform_vec3(&Vector3f::new(1.0, 0.0, 0.0))
            .into();
        let material_id = quad.material;

        let batch = match self.pending_batches.last_mut() {
            Some(batch) if batch.material_id == material_id => batch,
            _ => {
                self.pending_batches.push(PendingBatch::new(material_id));
                // SAFETY: We just added a batch to the pending batch list
                unsafe { self.pending_batches.last_mut().unwrap_unchecked() }
            }
        };

        batch.vertices.extend_from_slice(&[
            Vertex {
                position: top_left,
                texture_coordinates: [0.0, 1.0],
            },
            Vertex {
                position: bottom_left,
                texture_coordinates: [0.0, 0.0],
            },
            Vertex {
                position: bottom_right,
                texture_coordinates: [1.0, 0.0],
            },
            Vertex {
                position: bottom_right,
                texture_coordinates: [1.0, 0.0],
            },
            Vertex {
                position: top_right,
                texture_coordinates: [1.0, 1.0],
            },
            Vertex {
                position: top_left,
                texture_coordinates: [0.0, 1.0],
            },
        ]);
    }
}

impl RenderPass for Pass {
    fn prepare(&mut self, storage: &Storage) {
        let gfx = storage
            .resource::<GraphicsState>()
            .expect("Graphics state should be present");

        for (sprite, transform) in storage.query::<(&Sprite, &Transform)>().iter() {
            self.queue_quad_2d(&Quad2d {
                transform: transform.clone(),
                material: sprite
                    .material
                    .unwrap_or(gfx.placeholder_material_id.unwrap()),
            });
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
                material_id: batch.material_id,
            });
        }
    }

    fn execute(
        &self,
        gfx: &mut GraphicsState,
        pipelines: &RenderPipelines,
        encoder: &mut wgpu::CommandEncoder,
        surface_texture_view: &wgpu::TextureView,
        _storage: &Storage,
    ) {
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

        let pipeline = pipelines.get("pass_2d_pipeline");
        rpass.set_pipeline(pipeline);

        for batch in &self.batches_metadata {
            rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            let material = if let Some(material) = gfx.material_cache.get(batch.material_id) {
                material
            } else {
                gfx.material_cache
                    .get(gfx.placeholder_material_id.unwrap())
                    .expect("Placeholder material should be present in the material cache")
            };
            rpass.set_bind_group(0, material.bind_group(), &[]);
            rpass.draw(batch.start_vertex_index..batch.end_vertex_index, 0..1);
        }
    }
}

pub fn create_pass_2d_pipeline(
    device: &wgpu::Device,
    material_bind_group_layout: &wgpu::BindGroupLayout,
    surface_texture_format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let shader_module = device.create_shader_module(include_wgsl!("./pass_2d.wgsl"));

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&material_bind_group_layout],
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
