use log::debug;
use std::collections::HashMap;

use crate::{
    geometry::Vertex, material::MaterialCache, texture::DepthBufferTextureHandle, DrawCommand,
    RenderingContext,
};

#[derive(Clone, Copy, Debug)]
pub struct RenderTargetId(usize);

pub struct RenderGraph<'layout> {
    render_passes: Vec<RenderPass<'layout>>,
    render_targets: Vec<wgpu::TextureView>,
}

impl<'layout> RenderGraph<'layout> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            render_passes: vec![],
            render_targets: vec![],
        }
    }

    pub fn register_render_target(
        &mut self,
        render_target_texture_view: wgpu::TextureView,
    ) -> RenderTargetId {
        self.render_targets.push(render_target_texture_view);
        RenderTargetId(self.render_targets.len() - 1)
    }

    pub fn execute(
        &mut self,
        command_encoder: &mut wgpu::CommandEncoder,
        ctx: &mut RenderingContext,
    ) {
        for render_pass in &self.render_passes {
            let depth_stencil_attachment = if let Some(depth_buffer_texture_handle) =
                render_pass.depth_buffer_texture_handle
            {
                let depth_buffer_texture = ctx
                    .texture_cache
                    .depth_buffer_texture(depth_buffer_texture_handle);
                Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_buffer_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                })
            } else {
                None
            };

            let mut wgpu_render_pass =
                command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some(render_pass.identifier),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &self.render_targets[render_pass.render_targets[0].0],
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: true,
                        },
                    })],
                    depth_stencil_attachment,
                });

            let pass_identifier = render_pass.identifier.to_string();
            if !ctx.pipelines.contains_key(&pass_identifier) {
                debug!("Caching pipeline for pass {}", pass_identifier);
                let pipeline = Self::create_pipeline_for_pass(
                    &ctx.surface_configuration,
                    render_pass,
                    &ctx.device,
                    &ctx.shader_modules,
                );
                ctx.pipelines
                    .insert(render_pass.identifier.into(), pipeline);
            }

            wgpu_render_pass.set_pipeline(&ctx.pipelines[render_pass.identifier]);
            (render_pass.dispatch_fn)(
                &mut wgpu_render_pass,
                &render_pass.bind_groups,
                &ctx.vertex_buffers,
                &ctx.index_buffers,
                &ctx.draw_commands,
                &ctx.material_cache,
            );
        }
    }

    fn create_pipeline_for_pass(
        surface_configuration: &wgpu::SurfaceConfiguration,
        render_pass: &RenderPass,
        device: &wgpu::Device,
        shader_modules: &HashMap<String, wgpu::ShaderModule>,
    ) -> wgpu::RenderPipeline {
        let shader_module = &shader_modules[render_pass.shader_identifier];
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("{}_pipeline_layout", render_pass.identifier)),
            bind_group_layouts: &render_pass.bind_group_layouts,
            push_constant_ranges: &[],
        });

        let vertex_state_buffers = if render_pass.has_vertex_buffer {
            vec![Vertex::buffer_layout()]
        } else {
            vec![]
        };

        let depth_stencil = if render_pass.depth_buffer_texture_handle.is_some() {
            Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            })
        } else {
            None
        };

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&format!("{}_pipeline", render_pass.identifier)),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader_module,
                entry_point: "vs_main",
                buffers: &vertex_state_buffers,
            },
            primitive: wgpu::PrimitiveState {
                topology: render_pass.primitive_topology,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: shader_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_configuration.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        })
    }
}

impl Default for RenderGraph<'_> {
    fn default() -> Self {
        Self::new()
    }
}

type BoxedRenderPassDispatchFn = Box<
    dyn for<'l> Fn(
        &mut wgpu::RenderPass<'l>,
        &[&'l wgpu::BindGroup],
        &'l [wgpu::Buffer],
        &'l [wgpu::Buffer],
        &[DrawCommand],
        &'l MaterialCache,
    ),
>;
pub struct RenderPass<'layout> {
    identifier: &'static str,
    shader_identifier: &'static str,
    render_targets: Vec<RenderTargetId>,
    dispatch_fn: BoxedRenderPassDispatchFn,
    primitive_topology: wgpu::PrimitiveTopology,
    bind_group_layouts: Vec<&'layout wgpu::BindGroupLayout>,
    bind_groups: Vec<&'layout wgpu::BindGroup>,
    depth_buffer_texture_handle: Option<DepthBufferTextureHandle>,
    has_vertex_buffer: bool,
}

impl<'layout> RenderPass<'layout> {
    #[allow(clippy::new_ret_no_self)]
    pub fn new<'a>(
        render_pass_identifier: &'static str,
        render_graph: &'a mut RenderGraph<'layout>,
    ) -> RenderPassBuilder<'a, 'layout> {
        RenderPassBuilder::new(render_pass_identifier, render_graph)
    }
}

pub struct RenderPassBuilder<'a, 'layout> {
    identifier: &'static str,
    render_graph: &'a mut RenderGraph<'layout>,
    shader_identifier: Option<&'static str>,
    render_targets: Vec<RenderTargetId>,
    primitive_topology: wgpu::PrimitiveTopology,
    bind_group_layouts: Vec<&'layout wgpu::BindGroupLayout>,
    bind_groups: Vec<&'layout wgpu::BindGroup>,
    has_vertex_buffer: bool,
    depth_buffer_texture_handle: Option<DepthBufferTextureHandle>,
}

impl<'a, 'layout> RenderPassBuilder<'a, 'layout> {
    pub fn new(identifier: &'static str, render_graph: &'a mut RenderGraph<'layout>) -> Self {
        Self {
            identifier,
            render_graph,
            shader_identifier: None,
            render_targets: vec![],
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            bind_group_layouts: vec![],
            bind_groups: vec![],
            has_vertex_buffer: true,
            depth_buffer_texture_handle: None,
        }
    }

    #[must_use]
    pub fn with_shader(mut self, shader_identifier: &'static str) -> Self {
        self.shader_identifier = Some(shader_identifier);
        self
    }

    #[must_use]
    pub fn with_depth_buffer(
        mut self,
        depth_buffer_texture_handle: DepthBufferTextureHandle,
    ) -> Self {
        self.depth_buffer_texture_handle = Some(depth_buffer_texture_handle);
        self
    }

    #[must_use]
    pub fn with_render_target(mut self, render_target: RenderTargetId) -> Self {
        self.render_targets.push(render_target);
        self
    }

    #[must_use]
    pub fn with_primitive_topology(mut self, primitive_topology: wgpu::PrimitiveTopology) -> Self {
        self.primitive_topology = primitive_topology;
        self
    }

    #[must_use]
    pub fn with_no_vertex_buffer(mut self) -> Self {
        self.has_vertex_buffer = false;
        self
    }

    #[must_use]
    pub fn with_bind_group_layout(
        mut self,
        bind_group_layout: &'layout wgpu::BindGroupLayout,
    ) -> Self {
        self.bind_group_layouts.push(bind_group_layout);
        self
    }

    #[must_use]
    pub fn with_bind_group(
        mut self,
        bind_group_layout: &'layout wgpu::BindGroupLayout,
        bind_group: &'layout wgpu::BindGroup,
    ) -> Self {
        self.bind_group_layouts.push(bind_group_layout);
        self.bind_groups.push(bind_group);
        self
    }

    pub fn dispatch<F>(self, dispatch_fn: F)
    where
        F: 'static
            + for<'l> Fn(
                &mut wgpu::RenderPass<'l>,
                &[&'l wgpu::BindGroup],
                &'l [wgpu::Buffer],
                &'l [wgpu::Buffer],
                &[DrawCommand],
                &'l MaterialCache,
            ),
    {
        self.render_graph.render_passes.push(RenderPass {
            identifier: self.identifier,
            shader_identifier: self.shader_identifier.expect("Missing shader identifier"),
            render_targets: self.render_targets,
            primitive_topology: self.primitive_topology,
            dispatch_fn: Box::new(dispatch_fn),
            bind_group_layouts: self.bind_group_layouts,
            bind_groups: self.bind_groups,
            depth_buffer_texture_handle: self.depth_buffer_texture_handle,
            has_vertex_buffer: self.has_vertex_buffer,
        });
    }
}
