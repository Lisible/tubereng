use std::rc::Rc;

use tubereng_assets::{Asset, AssetHandle, AssetLoader};
use wgpu::BindGroupLayoutDescriptor;

pub struct ShaderCache {
    shaders: Vec<Option<Shader>>,
}

impl ShaderCache {
    #[must_use]
    pub fn new() -> Self {
        const MAX_SHADER_COUNT: usize = 1024;
        let mut shaders = vec![];
        shaders.resize_with(MAX_SHADER_COUNT, || None);

        Self { shaders }
    }

    #[must_use]
    pub fn has(&self, shader: AssetHandle<ShaderAsset>) -> bool {
        self.shaders[shader.id()].is_some()
    }

    #[must_use]
    pub fn get(&self, shader: AssetHandle<ShaderAsset>) -> Option<&Shader> {
        self.shaders[shader.id()].as_ref()
    }

    pub fn load(
        &mut self,
        device: &wgpu::Device,
        handle: AssetHandle<ShaderAsset>,
        shader_asset: Rc<ShaderAsset>,
    ) {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(&format!("shader_{}", handle.id())),
            source: wgpu::ShaderSource::Wgsl(shader_asset.source.clone().into()),
        });
        self.shaders[handle.id()] = Some(Shader {
            shader_module: shader,
        });
    }
}

impl Default for ShaderCache {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct ShaderAsset {
    pub source: String,
}

impl Asset for ShaderAsset {
    type Loader = ShaderLoader;
}

pub struct ShaderLoader;
impl AssetLoader<ShaderAsset> for ShaderLoader {
    fn load(file_content: &[u8]) -> tubereng_assets::Result<ShaderAsset> {
        Ok(ShaderAsset {
            source: String::from_utf8_lossy(file_content).into(),
        })
    }
}

pub struct Shader {
    shader_module: wgpu::ShaderModule,
    metadata: ShaderMetadata,
}

impl Shader {
    pub fn shader_module(&self) -> &wgpu::ShaderModule {
        &self.shader_module
    }
}

pub struct ShaderMetadata {
    name: String,
    bind_groups: Vec<BindGroup>,
}

impl ShaderMetadata {
    pub fn bind_group_layout_descriptors(&self) -> &[wgpu::BindGroupLayoutDescriptor] {
        &self
            .bind_groups
            .iter()
            .map(|bind_group| bind_group.into())
            .collect::<Vec<_>>()
    }

    pub fn bind_groups(&self) -> &[BindGroup] {
        &self.bind_groups
    }

    pub fn new(shader_name: &str) -> Self {
        Self {
            name: shader_name.into(),
            bind_groups: vec![],
        }
    }
}

pub struct BindGroup {
    entries: Vec<BindGroupEntry>,
}

impl BindGroup {
    pub fn entries(&self) -> &[BindGroupEntry] {
        &self.entries
    }
}

impl From<&BindGroup> for wgpu::BindGroupLayoutDescriptor<'_> {
    fn from(value: &BindGroup) -> Self {
        wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &value
                .entries
                .iter()
                .map(|bind_group_entry| bind_group_entry.into())
                .collect::<Vec<_>>(),
        }
    }
}

pub struct BindGroupEntry {
    name: String,
    r#type: BindingType,
    visibility: ShaderStage,
}

impl From<&BindGroupEntry> for wgpu::BindGroupLayoutEntry {
    fn from(value: &BindGroupEntry) -> Self {
        todo!()
    }
}

impl BindGroupEntry {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn r#type(&self) -> BindingType {
        self.r#type
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BindingType {
    Uniform,
    Sampler,
    Texture,
    Storage,
}

pub enum ShaderStage {
    Vertex,
    Fragment,
    VertexFragment,
    Compute,
}

impl From<ShaderStage> for wgpu::ShaderStages {
    fn from(value: ShaderStage) -> Self {
        match value {
            Vertex => wgpu::ShaderStages::VERTEX,
            Fragment => wgpu::ShaderStages::FRAGMENT,
            VertexFragment => wgpu::ShaderStages::VERTEX_FRAGMENT,
            Compute => wgpu::ShaderStages::COMPUTE,
        }
    }
}
