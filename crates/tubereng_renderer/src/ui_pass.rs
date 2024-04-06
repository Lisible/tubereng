use std::collections::{hash_map::Entry, HashMap};

use tubereng_math::matrix::Matrix4f;
use wgpu::include_wgsl;

use crate::{font::Font, texture, Color, DrawCommand, WgpuState};

const MAX_VERTICES: wgpu::BufferAddress = 1000;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
    texture_coordinates: [f32; 2],
}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x2];

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
    font_atlas_bind_group_layout: wgpu::BindGroupLayout,
    render_data: RenderData,
}

impl UiPass {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let common_uniforms_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("common_uniforms_bind_group_layout"),
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

        let font_atlas_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("font_atlas_bind_group_layout"),
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

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ui_pass_pipeline_layout"),
            bind_group_layouts: &[
                &common_uniforms_bind_group_layout,
                &font_atlas_bind_group_layout,
            ],
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
        });

        let render_data = RenderData::new(device, queue, &common_uniforms_bind_group_layout);

        Self {
            pipeline,
            _common_uniforms_bind_group_layout: common_uniforms_bind_group_layout,
            font_atlas_bind_group_layout,
            render_data,
        }
    }

    pub(crate) fn prepare(
        &mut self,
        state: &WgpuState<'_>,
        texture_cache: &texture::Cache,
        default_font: &Font,
        commands: &Vec<DrawCommand>,
    ) {
        self.render_data.quad_batches.clear();
        for command in commands {
            match &command {
                DrawCommand::DrawUiQuad(DrawUiQuadCommand {
                    x,
                    y,
                    width,
                    height,
                    texture_rect,
                    color,
                }) => self.draw_ui_quad(
                    state,
                    texture_cache,
                    *x,
                    *y,
                    *width,
                    *height,
                    color,
                    texture_rect,
                    texture_cache.white(),
                ),
                DrawCommand::DrawUiText(DrawUiTextCommand { text, x, y, color }) => {
                    self.draw_ui_text(state, texture_cache, default_font, text, *x, *y, color);
                }
            }
        }
    }

    fn draw_ui_text(
        &mut self,
        state: &WgpuState<'_>,
        texture_cache: &texture::Cache,
        font: &Font,
        text: &str,
        x: f32,
        y: f32,
        color: &Color,
    ) {
        let letter_spacing = font.letter_spacing();
        let mut x = x;
        for char in text.chars() {
            let glyph = font.glyphs().get(&char).unwrap();
            let glyph_width = glyph.x1 - glyph.x0;
            let glyph_height = glyph.y1 - glyph.y0;
            self.draw_ui_quad(
                state,
                texture_cache,
                x,
                y,
                glyph_width,
                glyph_height,
                color,
                &texture::Rect {
                    x: glyph.x0,
                    y: glyph.y0,
                    width: glyph_width,
                    height: glyph_height,
                },
                font.texture_id(),
            );
            x += glyph_width + letter_spacing;
        }
    }

    fn create_new_quad_batch(
        &mut self,
        device: &wgpu::Device,
        texture_cache: &texture::Cache,
        start_vertex_offset: u32,
        texture_id: texture::Id,
    ) {
        if let Entry::Vacant(e) = self.render_data.texture_bind_groups.entry(texture_id) {
            let texture = texture_cache.get(texture_id).unwrap();
            let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            });

            let font_atlas_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.font_atlas_bind_group_layout,
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

            e.insert(font_atlas_bind_group);
        }

        self.render_data
            .quad_batches
            .push(QuadBatch::new(start_vertex_offset, texture_id));
    }

    fn draw_ui_quad(
        &mut self,
        state: &WgpuState<'_>,
        texture_cache: &texture::Cache,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: &Color,
        texture_rect: &texture::Rect,
        texture_id: texture::Id,
    ) {
        let batch = self.render_data.quad_batches.last_mut();
        let batch = if let Some(batch) = batch {
            let end_vertex_offset = batch.end_vertex_offset;
            if batch.texture_id == texture_id {
                batch
            } else {
                self.create_new_quad_batch(
                    &state.device,
                    texture_cache,
                    end_vertex_offset,
                    texture_id,
                );
                self.render_data.quad_batches.last_mut().unwrap()
            }
        } else {
            self.create_new_quad_batch(&state.device, texture_cache, 0, texture_id);
            self.render_data.quad_batches.last_mut().unwrap()
        };

        let texture = texture_cache.get(texture_id).unwrap();
        let texture_size = texture.size();

        let quad_u = texture_rect.x / texture_size.width as f32;
        let quad_v = texture_rect.y / texture_size.height as f32;
        let quad_texture_width = texture_rect.width / texture_size.width as f32;
        let quad_texture_height = texture_rect.height / texture_size.height as f32;
        let color = color.into();

        let vertices: [Vertex; 6] = [
            Vertex {
                position: [x, y, 0.0],
                color,
                texture_coordinates: [quad_u, quad_v],
            },
            Vertex {
                position: [x, y + height, 0.0],
                color,
                texture_coordinates: [quad_u, quad_v + quad_texture_height],
            },
            Vertex {
                position: [x + width, y + height, 0.0],
                color,
                texture_coordinates: [quad_u + quad_texture_width, quad_v + quad_texture_height],
            },
            Vertex {
                position: [x + width, y + height, 0.0],
                color,
                texture_coordinates: [quad_u + quad_texture_width, quad_v + quad_texture_height],
            },
            Vertex {
                position: [x + width, y, 0.0],
                color,
                texture_coordinates: [quad_u + quad_texture_width, quad_v],
            },
            Vertex {
                position: [x, y, 0.0],
                color,
                texture_coordinates: [quad_u, quad_v],
            },
        ];

        state.queue.write_buffer(
            &self.render_data.vertex_buffer,
            u64::from(batch.end_vertex_offset) * std::mem::size_of::<Vertex>() as u64,
            bytemuck::cast_slice(&vertices),
        );
        batch.end_vertex_offset += 6;
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

        for batch in &self.render_data.quad_batches {
            rpass.set_bind_group(
                1,
                &self.render_data.texture_bind_groups[&batch.texture_id],
                &[],
            );
            rpass.draw(batch.start_vertex_offset..batch.end_vertex_offset, 0..1);
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CommonUniforms {
    projection_matrix: [[f32; 4]; 4],
}

pub struct QuadBatch {
    start_vertex_offset: u32,
    end_vertex_offset: u32,
    texture_id: texture::Id,
}

impl QuadBatch {
    pub fn new(start_vertex_offset: u32, texture: texture::Id) -> Self {
        Self {
            start_vertex_offset,
            end_vertex_offset: start_vertex_offset,
            texture_id: texture,
        }
    }
}

struct RenderData {
    vertex_buffer: wgpu::Buffer,
    quad_batches: Vec<QuadBatch>,

    common_uniforms_bind_group: wgpu::BindGroup,
    _common_uniforms_buffer: wgpu::Buffer,
    texture_bind_groups: HashMap<texture::Id, wgpu::BindGroup>,
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
            quad_batches: vec![],
            common_uniforms_bind_group,
            _common_uniforms_buffer: common_uniforms_buffer,
            texture_bind_groups: HashMap::new(),
        }
    }
}

pub struct DrawUiQuadCommand {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) color: Color,
    pub(crate) texture_rect: texture::Rect,
}

pub struct DrawUiTextCommand {
    pub(crate) text: String,
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) color: Color,
}
