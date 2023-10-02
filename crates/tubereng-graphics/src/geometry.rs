use crate::{GraphicsError, Result};
use tubereng_assets::{Asset, AssetHandle, AssetLoader, AssetStore};
use tubereng_obj::OBJParser;
use wgpu::util::{BufferInitDescriptor, DeviceExt};

#[derive(Debug)]
pub struct ModelAsset {
    mesh_descriptions: Vec<MeshDescription>,
}

#[derive(Debug)]
pub struct MeshDescription {
    vertices: Vec<Vertex>,
    _indices: Option<Vec<usize>>,
}

impl Asset for ModelAsset {
    type Loader = ModelAssetLoader;
}

pub struct ModelAssetLoader;
impl AssetLoader<ModelAsset> for ModelAssetLoader {
    fn load(file_content: &[u8]) -> tubereng_assets::Result<ModelAsset> {
        let file_content = String::from_utf8_lossy(file_content);
        let obj_model = OBJParser::parse(file_content).unwrap();

        let mut vertices = vec![];
        for face in &obj_model.faces {
            for triplet in &face.triplets {
                let pos = &obj_model.geometric_vertices[triplet.geometric_vertex - 1];
                let uv = &obj_model.texture_vertices[triplet.texture_vertex.unwrap() - 1];
                vertices.push(Vertex {
                    position: [pos.x, pos.y, pos.z],
                    texture_coordinates: [uv.u, uv.v],
                });
            }
        }

        Ok(ModelAsset {
            mesh_descriptions: vec![MeshDescription {
                vertices,
                _indices: None,
            }],
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
    pub fn has(&self, handle: AssetHandle<ModelAsset>) -> bool {
        self.models[handle.id()].is_some()
    }

    #[must_use]
    pub fn get(&self, handle: AssetHandle<ModelAsset>) -> Option<&Model> {
        self.models[handle.id()].as_ref()
    }

    /// # Errors
    /// Mau fail if the model asset is not found in the asset store
    pub fn load(
        &mut self,
        model_asset_handle: AssetHandle<ModelAsset>,
        asset_store: &mut AssetStore,
        vertex_buffers: &mut Vec<wgpu::Buffer>,
        _index_buffers: &[wgpu::Buffer],
        device: &wgpu::Device,
    ) -> Result<()> {
        let model_asset = asset_store
            .get(model_asset_handle)
            .ok_or(GraphicsError::ModelAssetNotFound)?;

        let mut meshes = vec![];
        for mesh_description in &model_asset.mesh_descriptions {
            vertex_buffers.push(device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&mesh_description.vertices),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
            }));

            let vertex_buffer = vertex_buffers.len() - 1;

            meshes.push(Mesh {
                vertex_buffer,
                index_buffer: None,
                element_count: u32::try_from(mesh_description.vertices.len())
                    .map_err(|_| GraphicsError::InvalidMesh)?,
            });
        }

        self.models[model_asset_handle.id()] = Some(Model { meshes });
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
    pub element_count: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    texture_coordinates: [f32; 2],
}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

    #[must_use]
    pub fn buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}
