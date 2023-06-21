type ResourceId = usize;

pub enum Resource {
    RenderTarget(wgpu::TextureView),
}

pub struct RenderGraph<'a> {
    render_passes: Vec<RenderPass<'a>>,
}

impl RenderGraph<'_> {
    pub fn new() -> Self {
        Self {
            render_passes: vec![],
        }
    }

    pub fn execute(&mut self, encoder: &mut wgpu::CommandEncoder) {
        for pass in &mut self.render_passes {}
    }
}

pub struct RenderPass<'a> {
    identifier: &'static str,
    pipeline_identifier: &'static str,
    render_targets: Vec<ResourceId>,
    dispatch_fn: Box<dyn FnMut(&mut wgpu::RenderPass<'a>)>,
}

impl<'g> RenderPass<'g> {
    pub fn new<'a>(
        render_pass_identifier: &'static str,
        render_graph: &'a mut RenderGraph<'g>,
    ) -> RenderPassBuilder<'a, 'g> {
        RenderPassBuilder::new(render_pass_identifier, render_graph)
    }
}

pub struct RenderPassBuilder<'a, 'g> {
    identifier: &'static str,
    render_graph: &'a mut RenderGraph<'g>,
    pipeline_identifier: Option<&'static str>,
    render_targets: Vec<ResourceId>,
}

impl<'a, 'g> RenderPassBuilder<'a, 'g> {
    pub fn new(identifier: &'static str, render_graph: &'a mut RenderGraph<'g>) -> Self {
        Self {
            identifier,
            render_graph,
            pipeline_identifier: None,
            render_targets: vec![],
        }
    }

    pub fn with_pipeline(mut self, pipeline_identifier: &'static str) -> Self {
        self.pipeline_identifier = Some(pipeline_identifier);
        self
    }

    pub fn with_render_target(mut self, render_target: ResourceId) -> Self {
        self.render_targets.push(render_target);
        self
    }

    pub fn dispatch<F>(self, dispatch_fn: F)
    where
        F: 'static + FnMut(&mut wgpu::RenderPass<'g>),
    {
        self.render_graph.render_passes.push(RenderPass {
            identifier: self.identifier,
            pipeline_identifier: self.pipeline_identifier.unwrap(),
            render_targets: self.render_targets,
            dispatch_fn: Box::new(dispatch_fn),
        });
    }
}
