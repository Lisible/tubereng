use crate::{RenderingContext, Result};

use tubereng_assets::AssetStore;
use tubereng_ecs::entity::EntityStore;

pub mod default_pipeline;

pub trait RenderPipeline {
    type RenderPipelineSettings: Default;
    /// Creates a new `RenderPipeline`
    ///
    /// # Errors
    ///
    /// This function will return an error if the creation fails
    fn new(
        render_pipeline_settings: &Self::RenderPipelineSettings,
        ctx: &mut RenderingContext,
        asset_store: &mut AssetStore,
    ) -> Result<Self>
    where
        Self: std::marker::Sized;
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
