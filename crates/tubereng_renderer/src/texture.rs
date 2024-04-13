use std::ops::Deref;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id(usize);
impl Deref for Id {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Cache {
    infos: Vec<Info>,
    textures: Vec<wgpu::Texture>,
}

impl Cache {
    #[must_use]
    pub fn new() -> Self {
        Self {
            infos: vec![],
            textures: vec![],
        }
    }

    pub fn insert(&mut self, texture_info: Info, texture: wgpu::Texture) -> Id {
        self.infos.push(texture_info);
        self.textures.push(texture);
        Id(self.textures.len() - 1)
    }

    #[must_use]
    pub fn info(&self, id: Id) -> &Info {
        &self.infos[*id]
    }

    #[must_use]
    pub fn get(&self, id: Id) -> &wgpu::Texture {
        &self.textures[*id]
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Info {
    pub(crate) width: u32,
    pub(crate) height: u32,
}

impl Info {
    #[must_use]
    pub fn width(&self) -> u32 {
        self.width
    }
    #[must_use]
    pub fn height(&self) -> u32 {
        self.height
    }
}

pub struct Descriptor<'a> {
    pub data: &'a [u8],
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}
