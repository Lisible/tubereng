#![warn(clippy::pedantic)]

use std::{io::Read, mem::MaybeUninit};

use serde::{Deserialize, Serialize};
use serde_json::Value as JSONValue;

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

#[derive(Debug)]
pub struct Glb {
    header: GlbHeader,
    gltf: Gltf,
}

impl TryFrom<&[u8]> for Glb {
    type Error = GltfError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        GlbParser::parse(value)
    }
}

struct GlbParser;
impl GlbParser {
    const MAGIC_NUMBER: u32 = 0x4654_6C67;
    const HEADER_SIZE: usize = 12;
    const CHUNK_TYPE_JSON: u32 = 0x4E4F_534A;
    const CHUNK_TYPE_BIN: u32 = 0x004E_4942;

    pub fn parse<R>(mut read: R) -> Result<Glb, GltfError>
    where
        R: Read,
    {
        let header = Self::parse_header(&mut read)?;
        let gltf = Self::parse_gltf_json_chunk(&mut read)?;

        Ok(Glb { header, gltf })
    }

    fn parse_header<R>(mut reader: &mut R) -> Result<GlbHeader, GltfError>
    where
        R: Read,
    {
        let magic_number = read_next_u32(&mut reader).map_err(GltfError::GlbHeaderParseError)?;
        if magic_number != Self::MAGIC_NUMBER {
            return Err(GltfError::GlbInvalidMagicNumber);
        }
        let version = read_next_u32(&mut reader).map_err(GltfError::GlbHeaderParseError)?;
        let length = read_next_u32(&mut reader).map_err(GltfError::GlbHeaderParseError)?;
        Ok(GlbHeader { version, length })
    }

    fn parse_gltf_json_chunk<R>(mut read: &mut R) -> Result<Gltf, GltfError>
    where
        R: Read,
    {
        let chunk_length = read_next_u32(&mut read).map_err(GltfError::GlbChunkParseError)?;
        let chunk_type = read_next_u32(&mut read).map_err(GltfError::GlbChunkParseError)?;
        if chunk_type != Self::CHUNK_TYPE_JSON {
            return Err(GltfError::GlbWrongChunkType);
        }

        let mut raw_chunk_data = vec![0u8; chunk_length as usize];
        read.read_exact(&mut raw_chunk_data)
            .map_err(GltfError::GlbChunkParseError)?;
        let gltf_string =
            String::from_utf8(raw_chunk_data).map_err(GltfError::GlbInvalidUtf8Data)?;
        let gltf: Gltf =
            serde_json::from_str(&gltf_string).map_err(GltfError::GlbInvalidGltfJson)?;

        Ok(gltf)
    }
}

fn read_next_u32<R>(reader: &mut R) -> std::io::Result<u32>
where
    R: Read,
{
    let mut value = [0u8; 4];
    reader.read_exact(&mut value)?;
    Ok(u32::from_le_bytes(value))
}

#[derive(Debug)]
pub struct GlbHeader {
    version: u32,
    length: u32,
}

#[derive(Debug)]
pub struct GlbChunk<'data> {
    chunk_length: u32,
    chunk_type: u32,
    chunk_data: &'data [u8],
}

/// <https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#reference-gltf>
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Gltf {
    #[serde(default)]
    pub extensions_used: Vec<String>,
    #[serde(default)]
    pub extensions_required: Vec<String>,
    #[serde(default)]
    pub accessors: Vec<Accessor>,
    #[serde(default)]
    pub animations: Vec<Animation>,
    pub asset: Asset,
    #[serde(default)]
    pub buffers: Vec<Buffer>,
    #[serde(default)]
    pub buffer_views: Vec<BufferView>,
    #[serde(default)]
    pub cameras: Vec<Camera>,
    #[serde(default)]
    pub images: Vec<Image>,
    #[serde(default)]
    pub materials: Vec<Material>,
    #[serde(default)]
    pub meshes: Vec<Mesh>,
    #[serde(default)]
    pub nodes: Vec<Node>,
    #[serde(default)]
    pub samplers: Vec<Sampler>,
    pub scene: Option<usize>,
    #[serde(default)]
    pub scenes: Vec<Scene>,
    #[serde(default)]
    pub skins: Vec<Skin>,
    #[serde(default)]
    pub textures: Vec<Texture>,
    pub extensions: Option<Extension>,
    pub extras: Option<Extras>,
}

/// <https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#reference-texture>
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Texture {
    pub sampler: Option<usize>,
    pub source: Option<usize>,
    pub name: Option<String>,
    pub extensions: Option<Extension>,
    pub extras: Option<Extras>,
}

/// <https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#reference-skin>
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Skin {
    pub inverse_bind_matrices: Option<usize>,
    pub skeleton: Option<usize>,
    #[serde(default)]
    pub joints: Vec<usize>,
    pub name: Option<String>,
    pub extensions: Option<Extension>,
    pub extras: Option<Extras>,
}

/// <https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#reference-animation>
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Animation {
    #[serde(default)]
    pub channels: Vec<AnimationChannel>,
    #[serde(default)]
    pub samplers: Vec<AnimationSampler>,
    pub name: Option<String>,
    pub extensions: Option<Extension>,
    pub extras: Option<Extras>,
}

/// <https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#reference-animation-sampler>
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationSampler {
    pub input: usize,
    #[serde(default = "animation_sampler_default_interpolation")]
    pub interpolation: String,
    pub output: usize,
    pub extensions: Option<Extension>,
    pub extras: Option<Extras>,
}

fn animation_sampler_default_interpolation() -> String {
    "LINEAR".into()
}

/// <https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#reference-animation-channel>
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationChannel {
    pub sampler: usize,
    pub target: AnimationChannelTarget,
    pub extensions: Option<Extension>,
    pub extras: Option<Extras>,
}

/// <https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#reference-animation-channel-target>
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationChannelTarget {
    pub node: Option<usize>,
    pub path: String,
    pub extensions: Option<Extension>,
    pub extras: Option<Extras>,
}

/// <https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#reference-scene>
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scene {
    #[serde(default)]
    pub nodes: Vec<usize>,
    pub name: Option<String>,
    pub extensions: Option<Extension>,
    pub extras: Option<Extras>,
}

/// <https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#reference-sampler>
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sampler {
    pub mag_filter: Option<usize>,
    pub min_filter: Option<usize>,
    #[serde(default = "default_sampler_wrap")]
    pub wrap_s: usize,
    #[serde(default = "default_sampler_wrap")]
    pub wrap_t: usize,
    pub name: Option<String>,
    pub extensions: Option<Extension>,
    pub extras: Option<Extras>,
}

const fn default_sampler_wrap() -> usize {
    10497
}

/// <https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#reference-node>
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Node {
    pub camera: Option<usize>,
    #[serde(default)]
    pub children: Vec<usize>,
    pub skin: Option<usize>,
    #[serde(default = "default_node_matrix")]
    pub matrix: [f32; 16],
    pub mesh: Option<usize>,
    #[serde(default = "default_node_rotation")]
    pub rotation: [f32; 4],
    #[serde(default = "default_node_scale")]
    pub scale: [f32; 3],
    #[serde(default)]
    pub translation: [f32; 3],
    #[serde(default)]
    pub weights: Vec<f32>,
    pub name: Option<String>,
    pub extensions: Option<Extension>,
    pub extras: Option<Extras>,
}

#[rustfmt::skip]
const fn default_node_matrix() -> [f32; 16] {
    [
        1.0, 0.0, 0.0, 0.0, 
        0.0, 1.0, 0.0, 0.0, 
        0.0, 0.0, 1.0, 0.0, 
        0.0, 0.0, 0.0, 1.0,
    ]
}

const fn default_node_rotation() -> [f32; 4] {
    [0.0, 0.0, 0.0, 1.0]
}

const fn default_node_scale() -> [f32; 3] {
    [1.0, 1.0, 1.0]
}

/// <https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#reference-mesh>
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mesh {
    #[serde(default)]
    pub primitives: Vec<MeshPrimitive>,
    #[serde(default)]
    pub weights: Vec<f32>,
    pub name: Option<String>,
    pub extensions: Option<Extension>,
    pub extras: Option<Extras>,
}

/// <https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#reference-mesh-primitive>
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MeshPrimitive {
    pub attributes: JSONValue,
    pub indices: Option<usize>,
    pub material: Option<usize>,
    #[serde(default = "default_mesh_primitive_mode")]
    pub mode: usize,
    #[serde(default)]
    pub targets: Vec<JSONValue>,
    pub extensions: Option<Extension>,
    pub extras: Option<Extras>,
}

const fn default_mesh_primitive_mode() -> usize {
    4
}

/// <https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#reference-material>
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Material {
    pub name: Option<String>,
    pub extensions: Option<Extension>,
    pub extras: Option<Extras>,
    pub pbr_metallic_roughness: Option<MaterialPbrMetallicRoughness>,
    pub normal_texture: Option<MaterialNormalTextureInfo>,
    pub occlusion_texture: Option<MaterialOcclusionTextureInfo>,
    pub emissive_texture: Option<TextureInfo>,
    #[serde(default)]
    pub emissive_factor: [f32; 3],
    #[serde(default = "default_material_alpha_mode")]
    pub alpha_mode: String,
    #[serde(default = "default_material_alpha_cutoff")]
    pub alpha_cutoff: f32,
    #[serde(default)]
    pub double_sided: bool,
}

fn default_material_alpha_mode() -> String {
    "OPAQUE".into()
}
const fn default_material_alpha_cutoff() -> f32 {
    0.5
}

/// <https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#reference-material-pbrmetallicroughness>
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaterialPbrMetallicRoughness {
    #[serde(default = "default_material_pbr_metallic_roughness_base_color_factor")]
    pub base_color_factor: [f32; 4],
    pub base_color_texture: Option<TextureInfo>,
    #[serde(default = "default_material_pbr_metallic_roughness_metallic_factor")]
    pub metallic_factor: f32,
    #[serde(default = "default_material_pbr_metallic_roughness_roughness_factor")]
    pub roughness_factor: f32,
    pub metallic_roughness_texture: Option<TextureInfo>,
    pub extensions: Option<Extension>,
    pub extras: Option<Extras>,
}

const fn default_material_pbr_metallic_roughness_base_color_factor() -> [f32; 4] {
    [1.0, 1.0, 1.0, 1.0]
}

const fn default_material_pbr_metallic_roughness_metallic_factor() -> f32 {
    1.0
}

const fn default_material_pbr_metallic_roughness_roughness_factor() -> f32 {
    1.0
}

/// <https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#reference-textureinfo>
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureInfo {
    pub index: usize,
    #[serde(default)]
    pub tex_coord: usize,
    pub extensions: Option<Extension>,
    pub extras: Option<Extras>,
}

/// <https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#reference-material-pbrmetallicroughness>
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaterialNormalTextureInfo {
    pub index: usize,
    #[serde(default)]
    pub tex_coord: usize,
    #[serde(default = "default_material_normal_texture_info_scale")]
    pub scale: f32,
    pub extensions: Option<Extension>,
    pub extras: Option<Extras>,
}

const fn default_material_normal_texture_info_scale() -> f32 {
    1.0
}

/// <https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#reference-material-occlusiontextureinfo>
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaterialOcclusionTextureInfo {
    pub index: usize,
    #[serde(default)]
    pub tex_coord: usize,
    #[serde(default = "default_material_occlusion_texture_info_strength")]
    pub strength: f32,
    pub extensions: Option<Extension>,
    pub extras: Option<Extras>,
}

const fn default_material_occlusion_texture_info_strength() -> f32 {
    1.0
}

/// <https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#reference-image>
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    pub uri: Option<String>,
    pub mime_type: Option<String>,
    pub buffer_view: Option<usize>,
    pub name: Option<String>,
    pub extensions: Option<Extension>,
    pub extras: Option<Extras>,
}

/// <https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#reference-camera>
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Camera {
    pub orthographic: Option<CameraOrthographic>,
    pub perspective: Option<CameraPerspective>,
    pub r#type: String,
    pub name: Option<String>,
    pub extensions: Option<Extension>,
    pub extras: Option<Extras>,
}

/// <https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#reference-camera-perspective>
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CameraPerspective {
    pub aspect_ratio: Option<f32>,
    pub yfov: f32,
    pub zfar: Option<f32>,
    pub znear: f32,
    pub extensions: Option<Extension>,
    pub extras: Option<Extras>,
}

/// <https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#reference-camera-orthographic>
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CameraOrthographic {
    pub xmag: f32,
    pub ymag: f32,
    pub zfar: f32,
    pub znear: f32,
    pub extensions: Option<Extension>,
    pub extras: Option<Extras>,
}

/// <https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#reference-bufferview>
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BufferView {
    pub buffer: usize,
    #[serde(default)]
    pub byte_offset: usize,
    pub byte_length: usize,
    pub byte_stride: Option<usize>,
    pub target: Option<usize>,
    pub name: Option<String>,
    pub extensions: Option<Extension>,
    pub extras: Option<Extras>,
}

/// <https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#reference-buffer>
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Buffer {
    pub uri: Option<String>,
    pub byte_length: usize,
    pub name: Option<String>,
    pub extensions: Option<Extension>,
    pub extras: Option<Extras>,
}

/// <https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#reference-accessor>
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Accessor {
    pub buffer_view: Option<usize>,
    #[serde(default)]
    pub byte_offset: usize,
    pub component_type: usize,
    #[serde(default)]
    pub normalized: bool,
    pub count: usize,
    pub r#type: String,
    #[serde(default)]
    pub max: Vec<f32>,
    #[serde(default)]
    pub min: Vec<f32>,
    pub sparse: Option<AccessorSparse>,
    pub name: Option<String>,
    pub extensions: Option<Extension>,
    pub extras: Option<Extras>,
}

/// <https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#reference-accessor-sparse>
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessorSparse {
    pub count: usize,
    pub indices: AccessorSparseIndices,
    pub values: AccessorSparseValues,
    pub extensions: Option<Extension>,
    pub extras: Option<Extras>,
}

/// <https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#reference-accessor-sparse-indices>
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessorSparseIndices {
    pub buffer_view: usize,
    #[serde(default)]
    pub byte_offset: usize,
    pub component_type: usize,
    pub extensions: Option<Extension>,
    pub extras: Option<Extras>,
}

/// <https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#reference-accessor-sparse-values>
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessorSparseValues {
    pub buffer_view: usize,
    #[serde(default)]
    pub byte_offset: usize,
    pub extensions: Option<Extension>,
    pub extras: Option<Extras>,
}

/// <https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#reference-asset>
#[derive(Debug, Deserialize, Serialize)]
pub struct Asset {
    pub copyright: Option<String>,
    pub generator: Option<String>,
    pub version: String,
    pub min_version: Option<String>,
    pub extensions: Option<Extension>,
    pub extras: Option<Extras>,
}

type Extension = JSONValue;
type Extras = JSONValue;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_gltf() {
        let gltf_str = include_str!("./test_data/1.gltf");
        let gltf: Gltf = serde_json::from_str(gltf_str).unwrap();
        assert_eq!(&gltf.asset.version, "2.0");
        assert_eq!(
            &gltf.asset.generator.unwrap(),
            "Khronos glTF Blender I/O v3.3.36"
        );
        assert_eq!(gltf.scene.unwrap(), 0);
        assert_eq!(gltf.scenes.len(), 1);
        assert_eq!(gltf.materials.len(), 1);
        assert_eq!(gltf.nodes.len(), 2);
        assert_eq!(gltf.textures.len(), 1);
        assert_eq!(gltf.images.len(), 1);
    }
}
