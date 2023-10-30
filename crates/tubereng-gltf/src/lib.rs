#![warn(clippy::pedantic)]

use std::ops::Range;

use parsing::GlbParser;
use tubereng_assets::{Asset, AssetLoader};
use tubereng_core::Transform;
use tubereng_graphics::geometry::{Index, Vertex};
use tubereng_math::{matrix::Matrix4f, quaternion::Quaternion, vector::Vector3f};

#[derive(Debug)]
pub enum GltfError {
    GlbCannotFitHeader,
    GlbHeaderParseError(std::io::Error),
    GlbChunkParseError(std::io::Error),
    GlbInvalidMagicNumber,
    GlbWrongChunkType,
    GlbInvalidUtf8Data(std::string::FromUtf8Error),
    GlbInvalidGltfJson(serde_json::Error),
}

mod parsing;

pub struct Gltf {
    scenes: Vec<Scene>,
    nodes: Vec<Node>,
    meshes: Vec<Mesh>,
    default_scene: Option<usize>,
}

impl Gltf {
    #[must_use]
    pub fn scenes(&self) -> &[Scene] {
        &self.scenes
    }

    #[must_use]
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }
    #[must_use]
    pub fn meshes(&self) -> &[Mesh] {
        &self.meshes
    }
    #[must_use]
    pub fn default_scene(&self) -> usize {
        self.default_scene.unwrap_or(0)
    }
}

impl Asset for Gltf {
    type Loader = GltfLoader;
}

pub struct GltfLoader;
impl AssetLoader<Gltf> for GltfLoader {
    fn load(file_content: &[u8]) -> tubereng_assets::Result<Gltf> {
        let glb = GlbParser::parse(file_content).unwrap();
        Ok(Gltf::from(glb))
    }
}

impl From<parsing::Glb> for Gltf {
    #[allow(clippy::too_many_lines)]
    fn from(glb: parsing::Glb) -> Self {
        let default_scene = glb.gltf.scene;
        let scenes: Vec<_> = glb
            .gltf
            .scenes
            .into_iter()
            .map(parsing::Scene::into)
            .collect();

        let nodes: Vec<_> = glb
            .gltf
            .nodes
            .into_iter()
            .map(parsing::Node::into)
            .collect();

        let mut meshes = vec![];
        for mesh in &glb.gltf.meshes {
            for primitive in &mesh.primitives {
                let position_attribute_byte_range = buffer_byte_range_for_attribute(
                    primitive,
                    &glb.gltf.accessors,
                    &glb.gltf.buffer_views,
                    "POSITION",
                );
                let mut vertex_positions: Vec<[f32; 3]> = vec![];
                for byte_index in position_attribute_byte_range.step_by(12) {
                    vertex_positions.push([
                        f32::from_le_bytes(
                            glb.binary_data[byte_index..=byte_index + 3]
                                .try_into()
                                .unwrap(),
                        ),
                        f32::from_le_bytes(
                            glb.binary_data[byte_index + 4..=byte_index + 7]
                                .try_into()
                                .unwrap(),
                        ),
                        f32::from_le_bytes(
                            glb.binary_data[byte_index + 8..=byte_index + 11]
                                .try_into()
                                .unwrap(),
                        ),
                    ]);
                }
                let normal_attribute_byte_range = buffer_byte_range_for_attribute(
                    primitive,
                    &glb.gltf.accessors,
                    &glb.gltf.buffer_views,
                    "NORMAL",
                );
                let mut vertex_normals: Vec<[f32; 3]> = vec![];
                for byte_index in normal_attribute_byte_range.step_by(12) {
                    vertex_normals.push([
                        f32::from_le_bytes(
                            glb.binary_data[byte_index..=byte_index + 3]
                                .try_into()
                                .unwrap(),
                        ),
                        f32::from_le_bytes(
                            glb.binary_data[byte_index + 4..=byte_index + 7]
                                .try_into()
                                .unwrap(),
                        ),
                        f32::from_le_bytes(
                            glb.binary_data[byte_index + 8..=byte_index + 11]
                                .try_into()
                                .unwrap(),
                        ),
                    ]);
                }

                let texture_coordinate_attribute_byte_range = buffer_byte_range_for_attribute(
                    primitive,
                    &glb.gltf.accessors,
                    &glb.gltf.buffer_views,
                    "TEXCOORD_0",
                );
                let mut vertex_texture_coordinates: Vec<[f32; 2]> = vec![];
                for byte_index in texture_coordinate_attribute_byte_range.step_by(8) {
                    vertex_texture_coordinates.push([
                        f32::from_le_bytes(
                            glb.binary_data[byte_index..=byte_index + 3]
                                .try_into()
                                .unwrap(),
                        ),
                        f32::from_le_bytes(
                            glb.binary_data[byte_index + 4..=byte_index + 7]
                                .try_into()
                                .unwrap(),
                        ),
                    ]);
                }

                let index_accessor_index = primitive.indices.unwrap();
                let index_accessor = &glb.gltf.accessors[index_accessor_index];
                let index_buffer_view_index = index_accessor.buffer_view.unwrap();
                let index_buffer_view = &glb.gltf.buffer_views[index_buffer_view_index];
                let mut indices = vec![];
                for byte_index in (index_buffer_view.byte_offset
                    ..index_buffer_view.byte_offset + index_buffer_view.byte_length)
                    .step_by(2)
                {
                    indices.push(u16::from_le_bytes(
                        glb.binary_data[byte_index..=byte_index + 1]
                            .try_into()
                            .unwrap(),
                    ));
                }

                let vertices = (0..vertex_positions.len())
                    .map(|vertex_index| Vertex {
                        position: vertex_positions[vertex_index],
                        color: [1.0, 1.0, 1.0],
                        normal: vertex_normals[vertex_index],
                        texture_coordinates: vertex_texture_coordinates[vertex_index],
                    })
                    .collect();
                meshes.push(Mesh { vertices, indices });
            }
        }

        Self {
            scenes,
            nodes,
            meshes,
            default_scene,
        }
    }
}

fn buffer_byte_range_for_attribute(
    primitive: &parsing::MeshPrimitive,
    accessors: &[parsing::Accessor],
    buffer_views: &[parsing::BufferView],
    attribute_identifier: &str,
) -> Range<usize> {
    let position_accessor_index = primitive
        .attributes
        .get(attribute_identifier)
        .unwrap_or_else(|| panic!("No {attribute_identifier} attribute in GLTF mesh primitive"))
        .as_u64()
        .unwrap();

    let position_accessor = &accessors[usize::try_from(position_accessor_index).unwrap()];
    let position_buffer_view_index = position_accessor.buffer_view.unwrap();
    let position_buffer_view = &buffer_views[position_buffer_view_index];
    position_buffer_view.byte_offset
        ..position_buffer_view.byte_offset + position_buffer_view.byte_length
}

pub struct Scene {
    name: Option<String>,
    nodes: Vec<usize>,
}

impl Scene {
    #[must_use]
    pub fn name(&self) -> &Option<String> {
        &self.name
    }

    #[must_use]
    pub fn nodes(&self) -> &[usize] {
        &self.nodes
    }
}

impl From<parsing::Scene> for Scene {
    fn from(value: parsing::Scene) -> Self {
        Scene {
            name: value.name,
            nodes: value.nodes,
        }
    }
}

pub struct Node {
    name: Option<String>,
    children: Vec<usize>,
    transform: Transform,
    mesh: Option<usize>,
}

impl Node {
    #[must_use]
    pub fn name(&self) -> &Option<String> {
        &self.name
    }

    #[must_use]
    pub fn children(&self) -> &[usize] {
        &self.children
    }

    #[must_use]
    pub fn transform(&self) -> &Transform {
        &self.transform
    }

    #[must_use]
    pub fn mesh(&self) -> &Option<usize> {
        &self.mesh
    }
}

impl From<parsing::Node> for Node {
    fn from(value: parsing::Node) -> Self {
        let transform = if let Some(matrix) = value.matrix {
            Transform::from(Matrix4f::with_values(matrix))
        } else {
            let translation = value.translation.unwrap_or([0.0, 0.0, 0.0]);
            let scale = value.scale.unwrap_or([1.0, 1.0, 1.0]);
            let rotation = value.rotation.unwrap_or([0.0, 0.0, 0.0, 1.0]);
            let translation = Vector3f::from(translation);
            let scale = Vector3f::from(scale);
            let rotation = Quaternion::new(
                rotation[3],
                Vector3f::new(rotation[0], rotation[1], rotation[2]),
            );
            Transform {
                translation,
                scale,
                rotation,
            }
        };

        Self {
            name: value.name,
            children: value.children,
            transform,
            mesh: value.mesh,
        }
    }
}

pub struct Mesh {
    vertices: Vec<Vertex>,
    indices: Vec<Index>,
}

impl Mesh {
    #[must_use]
    pub fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }

    #[must_use]
    pub fn indices(&self) -> &[Index] {
        &self.indices
    }
}
