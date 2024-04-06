#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct Id(usize);

pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

pub struct Cache {
    textures: Vec<wgpu::Texture>,
    white_texture_id: Id,
}

impl Cache {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let mut textures = vec![];

        let white_texture =
            Self::generate_monochrome_texture(device, queue, [0xFF, 0xFF, 0xFF, 0xFF]);
        let white_texture_id = Id(textures.len());
        textures.push(white_texture);

        Self {
            textures,
            white_texture_id,
        }
    }

    pub fn store(&mut self, texture: wgpu::Texture) -> Id {
        self.textures.push(texture);
        Id(self.textures.len() - 1)
    }

    pub fn get(&self, texture_id: Id) -> Option<&wgpu::Texture> {
        self.textures.get(texture_id.0)
    }

    pub fn white(&self) -> Id {
        self.white_texture_id
    }

    fn generate_monochrome_texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        rgba: [u8; 4],
    ) -> wgpu::Texture {
        let texture_size = wgpu::Extent3d {
            width: 1,
            height: 1,
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
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            texture_size,
        );

        texture
    }
}
