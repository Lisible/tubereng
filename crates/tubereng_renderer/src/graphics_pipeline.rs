use tubereng_ecs::Storage;

use crate::GraphicsState;

pub struct GraphicsPipeline {
    passes: Vec<Box<dyn RenderPass>>,
}

impl GraphicsPipeline {
    #[must_use]
    pub fn builder() -> Builder {
        Builder::default()
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

#[derive(Default)]
pub struct Builder {
    passes: Vec<Box<dyn RenderPass>>,
}

impl Builder {
    pub fn add_pass<P>(&mut self, pass: P) -> &mut Self
    where
        P: 'static + RenderPass,
    {
        self.passes.push(Box::new(pass));
        self
    }

    pub fn build(&mut self) -> GraphicsPipeline {
        let mut passes = vec![];
        passes.append(&mut self.passes);
        GraphicsPipeline { passes }
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
