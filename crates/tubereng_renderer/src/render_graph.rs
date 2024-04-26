use tubereng_ecs::Storage;

use crate::{pass_2d, ClearPass, GraphicsState};

pub struct RenderGraph {
    passes: Vec<Box<dyn RenderPass>>,
}

impl RenderGraph {
    #[must_use]
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            passes: vec![Box::new(ClearPass), Box::new(pass_2d::Pass::new(device))],
        }
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
        encoder: &mut wgpu::CommandEncoder,
        surface_texture_view: &wgpu::TextureView,
        storage: &Storage,
    ) {
        for pass in &self.passes {
            pass.execute(graphics, encoder, surface_texture_view, storage);
        }
    }
}

pub trait RenderPass {
    fn prepare(&mut self, storage: &Storage);
    fn execute(
        &self,
        gfx: &mut GraphicsState,
        encoder: &mut wgpu::CommandEncoder,
        surface_texture_view: &wgpu::TextureView,
        storage: &Storage,
    );
}
