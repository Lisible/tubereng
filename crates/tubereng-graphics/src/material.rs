use crate::texture::TextureCache;
use tubereng_assets::{AssetHandle, AssetStore, RonAsset};

pub type MaterialId = usize;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct MaterialAsset {
    texture: String,
}

impl RonAsset for MaterialAsset {}

#[derive(Debug)]
pub struct Material {
    bind_group: wgpu::BindGroup,
}

impl Material {
    pub fn bind<'rpass>(&'rpass self, index: u32, rpass: &mut wgpu::RenderPass<'rpass>) {
        rpass.set_bind_group(index, &self.bind_group, &[]);
    }
}

const MAX_MATERIAL_COUNT: usize = 1024;
pub struct MaterialCache {
    materials: Vec<Option<Material>>,
}

impl MaterialCache {
    pub fn new(device: &wgpu::Device) -> Self {
        let mut materials = vec![];
        materials.resize_with(MAX_MATERIAL_COUNT, || None);

        Self { materials }
    }

    pub fn has(&self, handle: AssetHandle<MaterialAsset>) -> bool {
        self.materials[handle.id()].is_some()
    }

    pub fn get(&self, handle: AssetHandle<MaterialAsset>) -> &Material {
        &self.materials[handle.id()].as_ref().unwrap()
    }

    pub fn load(
        &mut self,
        material_asset_handle: AssetHandle<MaterialAsset>,
        asset_store: &mut AssetStore,
        texture_store: &mut TextureCache,
        material_bind_group_layout: &wgpu::BindGroupLayout,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        let texture = {
            let material = asset_store.get(material_asset_handle).unwrap();
            material.texture.clone()
        };
        let texture_asset_handle = asset_store.load(&texture).unwrap();
        let texture_asset = asset_store.get(texture_asset_handle).unwrap();
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

        self.materials[material_asset_handle.id()] = Some(Material {
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
        });
    }
}
