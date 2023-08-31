use std::{collections::HashMap, io::Cursor};

use image;
use tubereng_assets::{Asset, AssetError, AssetHandle, AssetLoader};

#[derive(Debug)]
pub struct Texture {
    image: image::DynamicImage,
}

impl Asset for Texture {
    type Loader = TextureLoader;
}

pub struct TextureLoader;
impl AssetLoader<Texture> for TextureLoader {
    fn load(file_content: &[u8]) -> tubereng_assets::Result<Texture> {
        // FIXME: Fix the error handling
        let image = image::io::Reader::new(Cursor::new(file_content))
            .with_guessed_format()
            .unwrap()
            .decode()
            .map_err(|e| {
                dbg!(e);
                AssetError::ImageDecodingFailed
            })?;
        Ok(Texture { image })
    }
}

pub struct TextureCache {
    textures: HashMap<AssetHandle<Texture>, wgpu::Texture>,
}

impl TextureCache {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
        }
    }

    pub fn has(&self, handle: AssetHandle<Texture>) -> bool {
        self.textures.contains_key(&handle)
    }

    pub fn get(&self, handle: AssetHandle<Texture>) -> &wgpu::Texture {
        &self.textures[&handle]
    }

    pub fn load_to_vram(
        &mut self,
        handle: AssetHandle<Texture>,
        texture: &Texture,
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

        self.textures.insert(handle, texture);
        &self.textures[&handle]
    }
}
