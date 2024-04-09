use crate::RenderPipelines;

pub struct RenderGraph {
    passes: Vec<Box<dyn RenderPass>>,
}

impl RenderGraph {
    #[must_use]
    pub fn new() -> Self {
        Self { passes: vec![] }
    }

    pub fn clear(&mut self) {
        self.passes.clear();
    }

    pub fn add_pass<P>(&mut self, pass: P)
    where
        P: 'static + RenderPass,
    {
        self.passes.push(Box::new(pass));
    }

    pub fn execute(
        &self,
        pipelines: &RenderPipelines,
        encoder: &mut wgpu::CommandEncoder,
        surface_texture_view: &wgpu::TextureView,
    ) {
        for pass in &self.passes {
            pass.execute(pipelines, encoder, surface_texture_view);
        }
    }
}

impl Default for RenderGraph {
    fn default() -> Self {
        Self::new()
    }
}

pub trait RenderPass {
    fn prepare(&mut self);
    fn execute(
        &self,
        pipelines: &RenderPipelines,
        encoder: &mut wgpu::CommandEncoder,
        surface_texture_view: &wgpu::TextureView,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SomePass;
    impl RenderPass for SomePass {
        fn prepare(&mut self) {}
        fn execute(
            &self,
            _pipelines: &RenderPipelines,
            _encoder: &mut wgpu::CommandEncoder,
            _surface_texture_view: &wgpu::TextureView,
        ) {
        }
    }

    #[test]
    fn add_pass() {
        let mut graph = RenderGraph::new();
        graph.add_pass(SomePass);
        assert_eq!(graph.passes.len(), 1);
    }
}
