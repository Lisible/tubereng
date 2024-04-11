use std::ops::Deref;

use tubereng_core::Transform;

use crate::{material, texture};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Id(usize);
impl Deref for Id {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Cache {
    meshes: Vec<GpuMesh>,
}

impl Cache {
    #[must_use]
    pub fn new() -> Self {
        Self { meshes: vec![] }
    }

    pub fn insert(&mut self, mesh: GpuMesh) -> Id {
        self.meshes.push(mesh);
        Id(self.meshes.len() - 1)
    }

    #[must_use]
    pub fn get(&self, id: Id) -> &GpuMesh {
        &self.meshes[*id]
    }
}

pub struct GpuMesh {
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) vertex_count: usize,
}

#[repr(C)]
#[derive(bytemuck::Zeroable, bytemuck::Pod, Debug, Copy, Clone)]
pub struct Vertex {
    pub(crate) position: [f32; 3],
    pub(crate) texture_coordinates: [f32; 2],
}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

    pub fn layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

pub struct Descriptor {
    pub(crate) vertices: Vec<Vertex>,
}

pub struct Quad2dTexture {
    pub(crate) texture_id: texture::Id,
    pub(crate) texture_rect: texture::Rect,
}
