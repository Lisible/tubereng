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
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[
                Vertex {
                    position: [0.5f32, -0.5f32, -0.5f32],
                    texture_coordinates: [0.0, 0.0],
                },
                Vertex {
                    position: [0.5f32, -0.5f32, 0.5f32],
                    texture_coordinates: [0.0, 1.0],
                },
                Vertex {
                    position: [-0.5f32, -0.5f32, 0.5f32],
                    texture_coordinates: [1.0, 1.0],
                },
                Vertex {
                    position: [-0.5f32, -0.5f32, -0.5f32],
                    texture_coordinates: [1.0, 1.0],
                },
                Vertex {
                    position: [0.5f32, 0.5f32, -0.5f32],
                    texture_coordinates: [1.0, 0.0],
                },
                Vertex {
                    position: [0.5f32, 0.5f32, 0.5f32],
                    texture_coordinates: [0.0, 0.0],
                },
                Vertex {
                    position: [-0.5f32, 0.5f32, 0.5f32],
                    texture_coordinates: [1.0, 0.0],
                },
                Vertex {
                    position: [-0.5f32, 0.5f32, -0.5f32],
                    texture_coordinates: [0.0, 1.0],
                },
            ]),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[
                4u16, 2, 0, 2, 7, 3, 6, 5, 7, 1, 7, 5, 0, 3, 1, 4, 1, 5, 4, 6, 2, 2, 6, 7, 6, 4, 5,
                1, 3, 7, 0, 2, 3, 4, 0, 1,
            ]),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::INDEX,
        });

        vertex_buffers.push(vertex_buffer);
        index_buffers.push(index_buffer);

        Self {
            meshes: vec![Mesh {
                vertex_buffer: vertex_buffers.len() - 1,
                index_buffer: index_buffers.len() - 1,
                element_count: 36,
            }],
        }
    }
}

pub struct Mesh {
    pub(crate) vertex_buffer: usize,
    pub(crate) index_buffer: usize,
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
