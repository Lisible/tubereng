use tubereng_math::matrix::Matrix4f;
use wgpu::include_wgsl;

use crate::{DrawCommand, WgpuState};

const MAX_VERTICES: wgpu::BufferAddress = 100;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x3];

    pub fn buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

pub struct UiPass {
    pipeline: wgpu::RenderPipeline,
    _common_uniforms_bind_group_layout: wgpu::BindGroupLayout,
    render_data: RenderData,
}

impl UiPass {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let common_uniforms_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("common_uniform_bind_group_layout"),
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

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ui_pass_pipeline_layout"),
            bind_group_layouts: &[&common_uniforms_bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader_module = device.create_shader_module(include_wgsl!("ui_pass.wgsl"));

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ui_pass_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[Vertex::buffer_layout()],
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
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        let render_data = RenderData::new(device, queue, &common_uniforms_bind_group_layout);

        Self {
            pipeline,
            _common_uniforms_bind_group_layout: common_uniforms_bind_group_layout,
            render_data,
        }
    }

    pub(crate) fn prepare(&mut self, state: &WgpuState<'_>, commands: &Vec<DrawCommand>) {
        self.render_data.quad_count = 0;
        for command in commands {
            let &DrawCommand::DrawUiQuad(DrawUiQuadCommand {
                x,
                y,
                width,
                height,
            }): &DrawCommand = command;

            let vertices: [Vertex; 6] = [
                Vertex {
                    position: [x, y, 0.0],
                },
                Vertex {
                    position: [x, y + height, 0.0],
                },
                Vertex {
                    position: [x + width, y + height, 0.0],
                },
                Vertex {
                    position: [x + width, y + height, 0.0],
                },
                Vertex {
                    position: [x + width, y, 0.0],
                },
                Vertex {
                    position: [x, y, 0.0],
                },
            ];

            state.queue.write_buffer(
                &self.render_data.vertex_buffer,
                u64::from(self.render_data.quad_count) * 6 * std::mem::size_of::<Vertex>() as u64,
                bytemuck::cast_slice(&vertices),
            );

            self.render_data.quad_count += 1;
        }
    }

    pub(crate) fn execute(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        surface_texture_view: &wgpu::TextureView,
    ) {
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ui_pass"),
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

        rpass.set_pipeline(&self.pipeline);
        rpass.set_vertex_buffer(0, self.render_data.vertex_buffer.slice(..));
        rpass.set_bind_group(0, &self.render_data.common_uniforms_bind_group, &[]);

        for quad in 0u32..self.render_data.quad_count {
            rpass.draw((quad * 6)..(quad * 6 + 6), 0..1);
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CommonUniforms {
    projection_matrix: [[f32; 4]; 4],
}

struct RenderData {
    vertex_buffer: wgpu::Buffer,
    quad_count: u32,

    common_uniforms_bind_group: wgpu::BindGroup,
    _common_uniforms_buffer: wgpu::Buffer,
}

impl RenderData {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        common_uniform_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ui_vertex_buffer"),
            size: MAX_VERTICES * std::mem::size_of::<Vertex>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let common_uniforms_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ui_pass_common_uniforms_buffer"),
            size: std::mem::size_of::<CommonUniforms>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let common_uniforms = CommonUniforms {
            projection_matrix: Matrix4f::new_orthographic(0.0, 800.0, 600.0, 0.0, 0.0, 100.0)
                .into(),
        };
        queue.write_buffer(
            &common_uniforms_buffer,
            0,
            bytemuck::cast_slice(&[common_uniforms]),
        );

        let common_uniforms_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ui_pass_common_uniforms_bind_group"),
            layout: common_uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: common_uniforms_buffer.as_entire_binding(),
            }],
        });

        Self {
            vertex_buffer,
            quad_count: 0,
            common_uniforms_bind_group,
            _common_uniforms_buffer: common_uniforms_buffer,
        }
    }
}

pub struct DrawUiQuadCommand {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) width: f32,
    pub(crate) height: f32,
}
