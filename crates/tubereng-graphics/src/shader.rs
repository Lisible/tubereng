use std::rc::Rc;

use tubereng_assets::{Asset, AssetHandle, AssetLoader};

pub struct ShaderCache {
    shaders: Vec<Option<Shader>>,
}

impl ShaderCache {
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
}

impl Shader {
    pub fn shader_module(&self) -> &wgpu::ShaderModule {
        &self.shader_module
    }
}
