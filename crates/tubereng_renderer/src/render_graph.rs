use tubereng_ecs::Storage;

use crate::{GraphicsState, RenderPipelines};

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

    pub fn prepare(&mut self, storage: &Storage) {
        for pass in &mut self.passes {
            pass.prepare(storage);
        }
    }

    pub fn execute(
        &self,
        graphics: &mut GraphicsState,
        pipelines: &RenderPipelines,
        encoder: &mut wgpu::CommandEncoder,
        surface_texture_view: &wgpu::TextureView,
        storage: &Storage,
    ) {
        for pass in &self.passes {
            pass.execute(graphics, pipelines, encoder, surface_texture_view, storage);
        }
    }
}

impl Default for RenderGraph {
    fn default() -> Self {
        Self::new()
    }
}

pub trait RenderPass {
    fn prepare(&mut self, storage: &Storage);
    fn execute(
        &self,
        gfx: &mut GraphicsState,
        pipelines: &RenderPipelines,
        encoder: &mut wgpu::CommandEncoder,
        surface_texture_view: &wgpu::TextureView,
        storage: &Storage,
    );
}

#[cfg(test)]
mod tests {

    use super::*;

    struct SomePass;
    impl RenderPass for SomePass {
        fn prepare(&mut self, _storage: &Storage) {}
        fn execute(
            &self,
            _gfx: &mut GraphicsState,
            _pipelines: &RenderPipelines,
            _encoder: &mut wgpu::CommandEncoder,
            _surface_texture_view: &wgpu::TextureView,
            _storage: &Storage,
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
