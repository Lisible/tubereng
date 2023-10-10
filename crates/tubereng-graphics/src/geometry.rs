use crate::{GraphicsError, Result};
use tubereng_assets::{Asset, AssetHandle, AssetLoader, AssetStore};
use tubereng_obj::OBJParser;
use wgpu::util::{BufferInitDescriptor, DeviceExt};

#[derive(Debug)]
pub struct MeshAsset {
    pub mesh_description: MeshDescription,
}

#[derive(Debug)]
pub struct MeshDescription {
    pub vertices: Vec<Vertex>,
    pub indices: Option<Vec<u16>>,
}

impl Asset for MeshAsset {
    type Loader = MeshAssetLoader;
}

pub struct MeshAssetLoader;
impl AssetLoader<MeshAsset> for MeshAssetLoader {
    fn load(file_content: &[u8]) -> tubereng_assets::Result<MeshAsset> {
        let file_content = String::from_utf8_lossy(file_content);
        let obj_model = OBJParser::parse(file_content).unwrap();

        let mut vertices = vec![];
        for face in &obj_model.faces {
            for triplet in &face.triplets {
                let pos = &obj_model.geometric_vertices[triplet.geometric_vertex - 1];

                let texture_coordinates = if let Some(texture_vertex_index) = triplet.texture_vertex
                {
                    let texture_vertex = &obj_model.texture_vertices[texture_vertex_index - 1];
                    [texture_vertex.u, texture_vertex.v]
                } else {
                    [0.0, 0.0]
                };

                let normal = if let Some(vertex_normal_index) = triplet.vertex_normal {
                    let normal_vertex = &obj_model.vertex_normals[vertex_normal_index - 1];
                    [normal_vertex.i, normal_vertex.j, normal_vertex.k]
                } else {
                    [0.0, 0.0, 0.0]
                };

                vertices.push(Vertex {
                    position: [pos.x, pos.y, pos.z],
                    normal,
                    texture_coordinates,
                });
            }
        }

        Ok(MeshAsset {
            mesh_description: MeshDescription {
                vertices,
                indices: None,
            },
        })
    }
}

const MAX_MODEL_COUNT: usize = 1024;
pub struct ModelCache {
    models: Vec<Option<Model>>,
}

impl ModelCache {
    #[must_use]
    pub fn new() -> Self {
        let mut models = vec![];
        models.resize_with(MAX_MODEL_COUNT, || None);
        Self { models }
    }

    #[must_use]
    pub fn has(&self, handle: AssetHandle<MeshAsset>) -> bool {
        self.models[handle.id()].is_some()
    }

    #[must_use]
    pub fn get(&self, handle: AssetHandle<MeshAsset>) -> Option<&Model> {
        self.models[handle.id()].as_ref()
    }

    /// # Errors
    /// Mau fail if the model asset is not found in the asset store
    pub fn load(
        &mut self,
        mesh_asset_handle: AssetHandle<MeshAsset>,
        asset_store: &mut AssetStore,
        vertex_buffers: &mut Vec<wgpu::Buffer>,
        index_buffers: &mut Vec<wgpu::Buffer>,
        device: &wgpu::Device,
    ) -> Result<()> {
        let mesh_asset = asset_store
            .get(mesh_asset_handle)
            .ok_or(GraphicsError::ModelAssetNotFound)?;

        let mut meshes = vec![];

        vertex_buffers.push(device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&mesh_asset.mesh_description.vertices),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
        }));
        let vertex_buffer = vertex_buffers.len() - 1;
        let mut index_buffer = None;
        if let Some(mesh_indices) = &mesh_asset.mesh_description.indices {
            index_buffers.push(device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(mesh_indices),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::INDEX,
            }));
            index_buffer = Some(index_buffers.len() - 1);
        }

        meshes.push(Mesh {
            vertex_buffer,
            index_buffer,
            vertex_count: u32::try_from(mesh_asset.mesh_description.vertices.len())
                .map_err(|_| GraphicsError::InvalidMesh)?,
            element_count: if let Some(indices) = &mesh_asset.mesh_description.indices {
                u32::try_from(indices.len()).map_err(|_| GraphicsError::InvalidMesh)?
            } else {
                0
            },
        });

        self.models[mesh_asset_handle.id()] = Some(Model { meshes });
        Ok(())
    }
}

impl Default for ModelCache {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Model {
    pub meshes: Vec<Mesh>,
}

pub struct Mesh {
    pub vertex_buffer: usize,
    pub index_buffer: Option<usize>,
    pub vertex_count: u32,
    pub element_count: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub texture_coordinates: [f32; 2],
}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x2];

    #[must_use]
    pub fn buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}
