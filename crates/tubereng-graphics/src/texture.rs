use std::{collections::HashMap, io::Cursor};

use image;
use tubereng_assets::{Asset, AssetError, AssetHandle, AssetLoader};

#[derive(Debug)]
pub struct TextureAsset {
    pub(crate) image: image::DynamicImage,
}

impl Asset for TextureAsset {
    type Loader = TextureLoader;
}

pub struct TextureLoader;
impl AssetLoader<TextureAsset> for TextureLoader {
    fn load(file_content: &[u8]) -> tubereng_assets::Result<TextureAsset> {
        // FIXME: Fix the error handling
        let image = image::io::Reader::new(Cursor::new(file_content))
            .with_guessed_format()
            .unwrap()
            .decode()
            .map_err(|e| {
                dbg!(e);
                AssetError::ImageDecodingFailed
            })?;
        Ok(TextureAsset { image })
    }
}

const MAX_TEXTURE_COUNT: usize = 4096;

pub struct TextureCache {
    textures: Vec<Option<wgpu::Texture>>,
}

impl TextureCache {
    pub fn new() -> Self {
        let mut textures = vec![];
        textures.resize_with(MAX_TEXTURE_COUNT, || None);
        Self { textures }
    }

    pub fn has(&self, handle: AssetHandle<TextureAsset>) -> bool {
        self.textures[handle.id()].is_some()
    }

    pub fn get(&self, handle: AssetHandle<TextureAsset>) -> Option<&wgpu::Texture> {
        self.textures[handle.id()].as_ref()
    }

    pub fn load_to_vram(
        &mut self,
        handle: AssetHandle<TextureAsset>,
        texture: &TextureAsset,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> &wgpu::Texture {
        let rgba = texture.image.to_rgba8();
        use image::GenericImageView;
        let dimensions = texture.image.dimensions();
        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            texture_size,
        );

        self.textures[handle.id()] = Some(texture);
        self.textures[handle.id()].as_ref().unwrap()
    }
}
