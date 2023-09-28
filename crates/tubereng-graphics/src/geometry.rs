use tubereng_assets::{Asset, AssetLoader};
use tubereng_obj::OBJParser;
use wgpu::util::{BufferInitDescriptor, DeviceExt};

pub struct Model {
    pub meshes: Vec<Mesh>,
}

impl Model {
    pub fn new_cube(
        device: &wgpu::Device,
        vertex_buffers: &mut Vec<wgpu::Buffer>,
        index_buffers: &mut Vec<wgpu::Buffer>,
    ) -> Self {
        let obj_model = OBJParser::parse(include_str!("./cube.obj")).unwrap();
        let mut vertices = vec![];
        for face in &obj_model.faces {
            for triplet in &face.triplets {
                let geometric_vertex_index = triplet.geometric_vertex;
                let pos = &obj_model.geometric_vertices[geometric_vertex_index - 1];
                let texture_vertex_index = triplet.texture_vertex.unwrap();
                let uv = &obj_model.texture_vertices[texture_vertex_index - 1];
                vertices.push(Vertex {
                    position: [pos.x, pos.y, pos.z],
                    texture_coordinates: [uv.u, uv.v],
                });
            }
        }

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
        });

        vertex_buffers.push(vertex_buffer);

        Self {
            meshes: vec![Mesh {
                vertex_buffer: vertex_buffers.len() - 1,
                index_buffer: None,
                element_count: 36,
            }],
        }
    }
}

impl Asset for Model {
    type Loader = ModelLoader;
}

pub struct ModelLoader;
impl ModelLoader {
    fn parse_obj<S>(obj_file_content: S) -> tubereng_assets::Result<Model>
    where
        S: ToString,
    {
        let obj_file_content_string = obj_file_content.to_string();
        dbg!(obj_file_content_string);
        Ok(Model { meshes: vec![] })
    }
}

impl AssetLoader<Model> for ModelLoader {
    fn load(file_content: &[u8]) -> tubereng_assets::Result<Model> {
        let file_content = String::from_utf8_lossy(file_content);
        Self::parse_obj(file_content)
    }
}

pub struct Mesh {
    pub(crate) vertex_buffer: usize,
    pub(crate) index_buffer: Option<usize>,
    pub(crate) element_count: u32,
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
