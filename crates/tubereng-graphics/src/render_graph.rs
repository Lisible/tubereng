use log::debug;
use std::collections::HashMap;

use crate::{geometry::Vertex, material::MaterialCache, DrawCommand, RenderingContext};

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
        bind_groups: &[&wgpu::BindGroup],
        ctx: &mut RenderingContext,
    ) {
        for render_pass in &self.render_passes {
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
                    depth_stencil_attachment: None,
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
            for (draw_command_index, draw_command) in ctx.draw_commands.iter().enumerate() {
                wgpu_render_pass
                    .set_vertex_buffer(0, ctx.vertex_buffers[draw_command.vertex_buffer].slice(..));

                if let Some(index_buffer) = draw_command.index_buffer {
                    wgpu_render_pass.set_index_buffer(
                        ctx.index_buffers[index_buffer].slice(..),
                        wgpu::IndexFormat::Uint16,
                    );
                }

                (render_pass.dispatch_fn)(
                    &mut wgpu_render_pass,
                    bind_groups,
                    draw_command_index,
                    draw_command,
                    &ctx.material_cache,
                );
            }
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

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&format!("{}_pipeline", render_pass.identifier)),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader_module,
                entry_point: "vs_main",
                buffers: &[Vertex::buffer_layout()],
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
            depth_stencil: None,
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
        usize,
        &DrawCommand,
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
        }
    }

    #[must_use]
    pub fn with_shader(mut self, shader_identifier: &'static str) -> Self {
        self.shader_identifier = Some(shader_identifier);
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
    pub fn with_bind_group_layout(
        mut self,
        bind_group_layout: &'layout wgpu::BindGroupLayout,
    ) -> Self {
        self.bind_group_layouts.push(bind_group_layout);
        self
    }

    pub fn dispatch<F>(self, dispatch_fn: F)
    where
        F: 'static
            + for<'l> Fn(
                &mut wgpu::RenderPass<'l>,
                &[&'l wgpu::BindGroup],
                usize,
                &DrawCommand,
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
        });
    }
}
