use std::collections::HashMap;

use image::GenericImageView;

use crate::texture::{Cache, Id};

pub type Glyphs = HashMap<char, Glyph>;
pub struct Font {
    texture_id: Id,
    letter_spacing: f32,
    glyphs: Glyphs,
}

impl Font {
    pub fn new(texture_id: Id) -> Self {
        Self {
            texture_id,
            letter_spacing: 1.0,
            glyphs: HashMap::new(),
        }
    }

    fn add_glyph(&mut self, glyph: char, x: f32, y: f32, width: f32, height: f32) {
        self.glyphs.insert(
            glyph,
            Glyph {
                x0: x,
                y0: y,
                x1: width,
                y1: height,
            },
        );
    }
    pub fn texture_id(&self) -> Id {
        self.texture_id
    }

    pub fn glyphs(&self) -> &Glyphs {
        &self.glyphs
    }

    pub fn letter_spacing(&self) -> f32 {
        self.letter_spacing
    }
}

pub struct Glyph {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
}

pub(crate) fn create_default_font(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture_cache: &mut Cache,
) -> Font {
    let atlas_image_data = include_bytes!("../res/default_engine_font.png");
    let atlas_image =
        image::load_from_memory(atlas_image_data).expect("Couldn't load default font image");
    let atlas_image_rgba8_data = atlas_image.to_rgba8();
    let atlas_image_dimensions = atlas_image.dimensions();
    let texture_size = wgpu::Extent3d {
        width: atlas_image_dimensions.0,
        height: atlas_image_dimensions.1,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("default_engine_font_atlas_texture"),
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
            aspect: wgpu::TextureAspect::All,
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
        },
        &atlas_image_rgba8_data,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * atlas_image_dimensions.0),
            rows_per_image: Some(atlas_image_dimensions.1),
        },
        texture_size,
    );

    let texture_id = texture_cache.store(texture);
    let mut font = Font::new(texture_id);
    font.add_glyph('a', 2.0, 1.0, 14.0, 15.0);
    font.add_glyph('b', 19.0, 1.0, 30.0, 15.0);
    font.add_glyph('c', 35.0, 1.0, 46.0, 15.0);
    font.add_glyph('d', 51.0, 1.0, 62.0, 15.0);
    font.add_glyph('e', 66.0, 1.0, 77.0, 15.0);
    font.add_glyph('f', 82.0, 1.0, 93.0, 15.0);
    font.add_glyph('g', 98.0, 1.0, 109.0, 15.0);
    font.add_glyph('h', 114.0, 1.0, 125.0, 15.0);
    font.add_glyph('i', 134.0, 1.0, 138.0, 15.0);
    font.add_glyph('j', 146.0, 1.0, 158.0, 15.0);
    font.add_glyph('k', 162.0, 1.0, 173.0, 15.0);
    font.add_glyph('l', 178.0, 1.0, 189.0, 15.0);
    font.add_glyph('m', 194.0, 1.0, 205.0, 15.0);
    font.add_glyph('n', 2.0, 17.0, 14.0, 31.0);
    font.add_glyph('o', 19.0, 17.0, 30.0, 31.0);
    font.add_glyph('p', 35.0, 17.0, 46.0, 31.0);
    font.add_glyph('q', 51.0, 17.0, 62.0, 31.0);
    font.add_glyph('r', 66.0, 17.0, 77.0, 31.0);
    font.add_glyph('s', 82.0, 17.0, 93.0, 31.0);
    font.add_glyph('t', 98.0, 17.0, 109.0, 31.0);
    font.add_glyph('u', 114.0, 17.0, 125.0, 31.0);
    font.add_glyph('v', 134.0, 17.0, 140.0, 31.0);
    font.add_glyph('w', 146.0, 17.0, 158.0, 31.0);
    font.add_glyph('x', 162.0, 17.0, 173.0, 31.0);
    font.add_glyph('y', 178.0, 17.0, 189.0, 31.0);
    font.add_glyph('z', 194.0, 17.0, 205.0, 31.0);
    font.add_glyph('A', 2.0, 1.0, 14.0, 15.0);
    font.add_glyph('B', 19.0, 1.0, 30.0, 15.0);
    font.add_glyph('C', 35.0, 1.0, 46.0, 15.0);
    font.add_glyph('D', 51.0, 1.0, 62.0, 15.0);
    font.add_glyph('E', 66.0, 1.0, 77.0, 15.0);
    font.add_glyph('F', 82.0, 1.0, 93.0, 15.0);
    font.add_glyph('G', 98.0, 1.0, 109.0, 15.0);
    font.add_glyph('H', 114.0, 1.0, 125.0, 15.0);
    font.add_glyph('I', 134.0, 1.0, 138.0, 15.0);
    font.add_glyph('J', 146.0, 1.0, 158.0, 15.0);
    font.add_glyph('K', 162.0, 1.0, 173.0, 15.0);
    font.add_glyph('L', 178.0, 1.0, 189.0, 15.0);
    font.add_glyph('M', 194.0, 1.0, 205.0, 15.0);
    font.add_glyph('N', 2.0, 17.0, 14.0, 31.0);
    font.add_glyph('O', 19.0, 17.0, 30.0, 31.0);
    font.add_glyph('P', 35.0, 17.0, 46.0, 31.0);
    font.add_glyph('Q', 51.0, 17.0, 62.0, 31.0);
    font.add_glyph('R', 66.0, 17.0, 77.0, 31.0);
    font.add_glyph('S', 82.0, 17.0, 93.0, 31.0);
    font.add_glyph('T', 98.0, 17.0, 109.0, 31.0);
    font.add_glyph('U', 114.0, 17.0, 125.0, 31.0);
    font.add_glyph('V', 134.0, 17.0, 140.0, 31.0);
    font.add_glyph('W', 146.0, 17.0, 158.0, 31.0);
    font.add_glyph('X', 162.0, 17.0, 173.0, 31.0);
    font.add_glyph('Y', 178.0, 17.0, 189.0, 31.0);
    font.add_glyph('Z', 194.0, 17.0, 205.0, 31.0);
    font
}
