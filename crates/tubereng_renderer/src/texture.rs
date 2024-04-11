use std::ops::Deref;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Id(usize);
impl Deref for Id {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Cache {
    textures: Vec<wgpu::Texture>,
}

impl Cache {
    #[must_use]
    pub fn new() -> Self {
        Self { textures: vec![] }
    }

    pub fn insert(&mut self, texture: wgpu::Texture) -> Id {
        self.textures.push(texture);
        Id(self.textures.len() - 1)
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

pub struct Descriptor<'a> {
    pub data: &'a [u8],
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}
