use crate::{
    wgsl::{self, AddressSpace, Variable, VariableKind},
    Result,
};
use std::{collections::HashMap, rc::Rc};
use tubereng_assets::{Asset, AssetHandle, AssetLoader};

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

    /// Loads a shader into the cache
    ///
    /// # Errors
    /// This might return an Err if the shader metadata cannot be created
    pub fn load(
        &mut self,
        device: &wgpu::Device,
        handle: AssetHandle<ShaderAsset>,
        shader_asset: &Rc<ShaderAsset>,
    ) -> Result<()> {
        let shader_name = &format!("shader_{}", handle.id());
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(shader_name),
            source: wgpu::ShaderSource::Wgsl(shader_asset.source.clone().into()),
        });

        let metadata = ShaderMetadata::new(shader_name, &shader_asset.source)?;
        let mut bind_group_layouts = vec![];
        for group in 0..metadata.bind_group_layout_entries().len() {
            bind_group_layouts.push(device.create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &metadata.bind_group_layout_entries()[group],
                },
            ));
        }

        self.shaders[handle.id()] = Some(Shader {
            shader_module,
            bind_group_layouts,
            metadata,
        });
        Ok(())
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
    bind_group_layouts: Vec<wgpu::BindGroupLayout>,
    metadata: ShaderMetadata,
}

impl Shader {
    #[must_use]
    pub fn shader_module(&self) -> &wgpu::ShaderModule {
        &self.shader_module
    }

    #[must_use]
    pub fn bind_group_layouts(&self) -> Vec<&wgpu::BindGroupLayout> {
        self.bind_group_layouts.iter().collect()
    }

    #[must_use]
    pub fn metadata(&self) -> &ShaderMetadata {
        &self.metadata
    }
}

pub struct ShaderMetadata {
    name: String,
    parameters_metadata: HashMap<String, ParameterMetadata>,
    bind_group_layout_entries: Vec<Vec<wgpu::BindGroupLayoutEntry>>,
}

impl ShaderMetadata {
    /// Creates a new `ShaderMetadata` instance
    ///
    /// # Errors
    ///
    /// This function will return an error if the global variables extraction from the shader source fails
    pub fn new(shader_name: &str, source: &str) -> Result<Self> {
        let mut variables = wgsl::extract_global_variables_from_shader_source(source)?;
        variables.sort_by(|a, b| {
            a.attributes
                .group
                .cmp(&b.attributes.group)
                .then(a.attributes.binding.cmp(&b.attributes.binding))
        });

        let mut parameters_metadata = HashMap::new();
        let mut bind_group_entries_per_group = vec![];
        for variable in &variables {
            parameters_metadata.insert(
                variable.identifier.clone(),
                ParameterMetadata {
                    group: variable.attributes.group,
                    binding: variable.attributes.binding,
                },
            );

            let bind_group = variable.attributes.group as usize;
            if bind_group_entries_per_group.len() < bind_group + 1 {
                bind_group_entries_per_group.push(vec![]);
            }

            bind_group_entries_per_group[bind_group].push(wgpu::BindGroupLayoutEntry {
                binding: variable.attributes.binding,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: variable.binding_type(),
                count: None,
            });
        }

        Ok(ShaderMetadata {
            name: shader_name.to_string(),
            bind_group_layout_entries: bind_group_entries_per_group,
            parameters_metadata,
        })
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn parameter_metadata(&self, parameter_identifier: &str) -> &ParameterMetadata {
        &self.parameters_metadata[parameter_identifier]
    }

    #[must_use]
    pub fn bind_group_layout_entries(&self) -> &[Vec<wgpu::BindGroupLayoutEntry>] {
        &self.bind_group_layout_entries
    }
}

pub struct ParameterMetadata {
    group: u32,
    binding: u32,
}

impl ParameterMetadata {
    #[must_use]
    pub fn group(&self) -> u32 {
        self.group
    }

    #[must_use]
    pub fn binding(&self) -> u32 {
        self.binding
    }
}

#[derive(Clone)]
pub struct BindGroup {
    entries: Vec<BindGroupEntry>,
}

impl BindGroup {
    #[must_use]
    pub fn entries(&self) -> &[BindGroupEntry] {
        &self.entries
    }
}

#[derive(Clone)]
pub struct BindGroupEntry {
    name: String,
    binding_type: BindingType,
    visibility: ShaderStage,
}

impl BindGroupEntry {
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn binding_type(&self) -> BindingType {
        self.binding_type
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BindingType {
    Uniform,
    Sampler,
    Texture,
    Storage,
}

#[derive(Debug, Clone, Copy)]
pub enum ShaderStage {
    Vertex,
    Fragment,
    VertexFragment,
    Compute,
}

impl From<ShaderStage> for wgpu::ShaderStages {
    fn from(value: ShaderStage) -> Self {
        match value {
            ShaderStage::Vertex => wgpu::ShaderStages::VERTEX,
            ShaderStage::Fragment => wgpu::ShaderStages::FRAGMENT,
            ShaderStage::VertexFragment => wgpu::ShaderStages::VERTEX_FRAGMENT,
            ShaderStage::Compute => wgpu::ShaderStages::COMPUTE,
        }
    }
}

impl Variable {
    fn binding_type(&self) -> wgpu::BindingType {
        if let Some(address_space) = self.address_space {
            if address_space == AddressSpace::Storage {
                return wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                };
            }
        }

        if self.kind == VariableKind::Texture {
            return wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            };
        } else if self.kind == VariableKind::Sampler {
            return wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering);
        }

        let has_dynamic_offset = self.identifier.starts_with("dyn_");
        wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset,
            min_binding_size: None,
        }
    }
}
