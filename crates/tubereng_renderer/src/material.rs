use std::ops::Deref;

use crate::texture;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Id(usize);
impl Deref for Id {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Material {
    pub(crate) bind_group: wgpu::BindGroup,
}

impl Material {
    #[must_use]
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

pub struct Descriptor {
    pub base_color: texture::Id,
    pub region: texture::Rect,
}

pub struct Cache {
    material: Vec<Material>,
}

impl Cache {
    #[must_use]
    pub fn new() -> Self {
        Self { material: vec![] }
    }

    pub fn insert(&mut self, material: Material) -> Id {
        self.material.push(material);
        Id(self.material.len() - 1)
    }

    #[must_use]
    pub fn get(&self, id: Id) -> Option<&Material> {
        self.material.get(*id)
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}
