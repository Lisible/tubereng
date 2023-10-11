use crate::{
    shader::{ShaderAsset, ShaderCache},
    texture::TextureCache,
    GraphicsError, Result,
};
use tubereng_assets::{AssetHandle, AssetStore, RonAsset};

pub type MaterialId = usize;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub enum MaterialAsset {
    PbrMaterial(PbrMaterialAsset),
    ShaderMaterial(ShaderMaterialAsset),
}

impl RonAsset for MaterialAsset {}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ShaderMaterialAsset {
    pub shader: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct PbrMaterialAsset {
    texture: String,
}

#[derive(Debug)]
pub enum Material {
    PbrMaterial(PbrMaterial),
    ShaderMaterial(ShaderMaterial),
}

#[derive(Debug)]
pub struct PbrMaterial {
    bind_group: wgpu::BindGroup,
}

#[derive(Debug)]
pub struct ShaderMaterial {
    pub shader: AssetHandle<ShaderAsset>,
}

impl PbrMaterial {
    pub fn bind<'rpass>(&'rpass self, index: u32, rpass: &mut wgpu::RenderPass<'rpass>) {
        rpass.set_bind_group(index, &self.bind_group, &[]);
    }
}

const MAX_MATERIAL_COUNT: usize = 1024;
pub struct MaterialCache {
    materials: Vec<Option<Material>>,
}

impl MaterialCache {
    #[must_use]
    pub fn new(_device: &wgpu::Device) -> Self {
        let mut materials = vec![];
        materials.resize_with(MAX_MATERIAL_COUNT, || None);

        Self { materials }
    }

    #[must_use]
    pub fn has(&self, handle: AssetHandle<MaterialAsset>) -> bool {
        self.materials[handle.id()].is_some()
    }

    #[must_use]
    pub fn get(&self, handle: AssetHandle<MaterialAsset>) -> Option<&Material> {
        self.materials[handle.id()].as_ref()
    }

    /// # Errors
    /// This method might fail if an error occurs while loading the material
    pub fn load(
        &mut self,
        material_asset_handle: AssetHandle<MaterialAsset>,
        asset_store: &mut AssetStore,
        texture_store: &mut TextureCache,
        shader_cache: &mut ShaderCache,
        material_bind_group_layout: &wgpu::BindGroupLayout,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<()> {
        let material = asset_store
            .get(material_asset_handle)
            .ok_or(GraphicsError::MaterialAssetNotFound)?;
        match material.as_ref() {
            MaterialAsset::PbrMaterial(pbr_material) => {
                self.load_pbr_material(
                    device,
                    queue,
                    material_asset_handle,
                    asset_store,
                    texture_store,
                    material_bind_group_layout,
                    pbr_material,
                )?;
            }
            MaterialAsset::ShaderMaterial(shader_material) => {
                self.load_shader_material(
                    device,
                    material_asset_handle,
                    asset_store,
                    shader_cache,
                    shader_material,
                );
            }
        }

        Ok(())
    }

    fn load_pbr_material(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        material_asset_handle: AssetHandle<MaterialAsset>,
        asset_store: &mut AssetStore,
        texture_store: &mut TextureCache,
        material_bind_group_layout: &wgpu::BindGroupLayout,
        pbr_material: &PbrMaterialAsset,
    ) -> Result<()> {
        let texture_asset_handle = asset_store
            .load(&pbr_material.texture)
            .map_err(GraphicsError::AssetError)?;
        let texture_asset = asset_store
            .get(texture_asset_handle)
            .ok_or(GraphicsError::TextureAssetNotFound)?;
        let texture =
            texture_store.load_to_vram(texture_asset_handle, texture_asset, device, queue);
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        self.materials[material_asset_handle.id()] = Some(Material::PbrMaterial(PbrMaterial {
            bind_group: device.create_bind_group(&wgpu::BindGroupDescriptor {
                // TODO: give a label to the material bind group
                label: None,
                layout: material_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&texture_sampler),
                    },
                ],
            }),
        }));

        Ok(())
    }

    fn load_shader_material(
        &mut self,
        device: &wgpu::Device,
        material_asset_handle: AssetHandle<MaterialAsset>,
        asset_store: &mut AssetStore,
        shader_cache: &mut ShaderCache,
        shader_material: &ShaderMaterialAsset,
    ) {
        let shader_path = &shader_material.shader;
        let shader_asset_handle = asset_store.load::<ShaderAsset>(shader_path).unwrap();
        let shader_asset = asset_store.get(shader_asset_handle).unwrap();
        shader_cache.load(device, shader_asset_handle, shader_asset);

        self.materials[material_asset_handle.id()] =
            Some(Material::ShaderMaterial(ShaderMaterial {
                shader: shader_asset_handle,
            }));
    }
}
