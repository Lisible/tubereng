use std::io::Cursor;

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

#[derive(Debug, Copy, Clone)]
pub struct DepthBufferTextureHandle(usize);
pub struct DepthBufferTexture {
    pub(crate) label: String,
    pub(crate) texture: wgpu::Texture,
    pub(crate) view: wgpu::TextureView,
    pub(crate) sampler: wgpu::Sampler,
    pub(crate) recreate_on_window_resize: bool,
}

pub struct TextureCache {
    textures: Vec<Option<wgpu::Texture>>,
    depth_buffer_textures: Vec<DepthBufferTexture>,
}

impl TextureCache {
    #[must_use]
    pub fn new() -> Self {
        let mut textures = vec![];
        textures.resize_with(MAX_TEXTURE_COUNT, || None);

        Self {
            textures,
            depth_buffer_textures: vec![],
        }
    }

    #[must_use]
    pub fn has(&self, handle: AssetHandle<TextureAsset>) -> bool {
        self.textures[handle.id()].is_some()
    }

    #[must_use]
    pub fn get(&self, handle: AssetHandle<TextureAsset>) -> Option<&wgpu::Texture> {
        self.textures[handle.id()].as_ref()
    }

    pub fn create_depth_texture(
        &mut self,
        device: &wgpu::Device,
        label: &str,
        width: u32,
        height: u32,
        recreate_on_window_resize: bool,
    ) -> DepthBufferTextureHandle {
        self.depth_buffer_textures
            .push(Self::create_depth_buffer_texture_impl(
                device,
                label,
                width,
                height,
                recreate_on_window_resize,
            ));
        DepthBufferTextureHandle(self.depth_buffer_textures.len() - 1)
    }

    fn create_depth_buffer_texture_impl(
        device: &wgpu::Device,
        label: &str,
        width: u32,
        height: u32,
        recreate_on_window_resize: bool,
    ) -> DepthBufferTexture {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture_descriptor = wgpu::TextureDescriptor {
            label: Some(&label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };

        let texture = device.create_texture(&texture_descriptor);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        DepthBufferTexture {
            label: label.to_string(),
            texture,
            view,
            sampler,
            recreate_on_window_resize,
        }
    }

    pub fn load_to_vram(
        &mut self,
        handle: AssetHandle<TextureAsset>,
        texture: &TextureAsset,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> &wgpu::Texture {
        use image::GenericImageView;
        let rgba = texture.image.to_rgba8();
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
        // SAFETY: We just assigned this texture so it is present
        unsafe { self.textures[handle.id()].as_ref().unwrap_unchecked() }
    }

    pub(crate) fn depth_buffer_texture(
        &self,
        depth_buffer_texture_handle: DepthBufferTextureHandle,
    ) -> &DepthBufferTexture {
        &self.depth_buffer_textures[depth_buffer_texture_handle.0]
    }

    pub(crate) fn on_window_resize(&mut self, device: &wgpu::Device, new_size: crate::WindowSize) {
        for depth_buffer_texture in self
            .depth_buffer_textures
            .iter_mut()
            .filter(|t| t.recreate_on_window_resize)
        {
            let new_depth_buffer_texture = Self::create_depth_buffer_texture_impl(
                device,
                &depth_buffer_texture.label,
                new_size.width,
                new_size.height,
                true,
            );
            *depth_buffer_texture = new_depth_buffer_texture;
        }
    }
}

impl Default for TextureCache {
    fn default() -> Self {
        Self::new()
    }
}
