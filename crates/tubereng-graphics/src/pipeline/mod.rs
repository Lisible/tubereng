use crate::{texture::TextureCache, RenderingContext, Result};
use std::collections::HashMap;

use tubereng_assets::AssetStore;
use tubereng_ecs::entity::EntityStore;

pub mod default_pipeline;

pub trait RenderPipeline {
    fn new(
        device: &wgpu::Device,
        surface_configuration: &wgpu::SurfaceConfiguration,
        texture_cache: &mut TextureCache,
        shader_modules: &mut HashMap<String, wgpu::ShaderModule>,
    ) -> Self;
    /// Prepares the render
    /// # Errors
    /// Returns an error if the preparation fails
    fn prepare(
        &mut self,
        rendering_context: &mut RenderingContext,
        entity_store: &EntityStore,
        asset_store: &mut AssetStore,
    ) -> Result<()>;

    /// Renders
    /// # Errors
    /// Returns an error if the render fails
    fn render(
        &mut self,
        command_encoder: &mut wgpu::CommandEncoder,
        view: wgpu::TextureView,
        rendering_context: &mut RenderingContext,
    ) -> Result<()>;
}
