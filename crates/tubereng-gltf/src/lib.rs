#![warn(clippy::pedantic)]

use serde::{Deserialize, Serialize};
use serde_json::Value as JSONValue;

/// <https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#reference-gltf>
#[derive(Debug, Deserialize)]
pub struct Gltf {
    pub extensions_used: Vec<String>,
    pub extensions_required: Vec<String>,
    pub accessors: Vec<Accessor>,
    pub animations: Vec<Animation>,
    pub asset: Asset,
    pub buffers: Vec<Buffer>,
    pub buffer_views: Vec<BufferView>,
    pub cameras: Vec<Camera>,
    pub images: Vec<Image>,
    pub materials: Vec<Material>,
    pub meshes: Vec<Mesh>,
    pub nodes: Vec<Node>,
    pub samplers: Vec<Sampler>,
    pub scene: Option<usize>,
    pub scenes: Vec<Scene>,
    pub skins: Vec<Skin>,
    pub textures: Vec<Texture>,
    pub extensions: Option<Extension>,
    pub extras: Option<Extras>,
}

/// <https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#reference-texture>
#[derive(Debug, Deserialize)]
pub struct Texture {
    pub sampler: Option<usize>,
    pub source: Option<usize>,
    pub name: Option<String>,
    pub extensions: Option<Extension>,
    pub extras: Option<Extras>,
}

/// <https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#reference-skin>
#[derive(Debug, Deserialize)]
pub struct Skin {
    pub inverse_bind_matrices: Option<usize>,
    pub skeleton: Option<usize>,
    pub joints: Vec<usize>,
    pub name: Option<String>,
    pub extensions: Option<Extension>,
    pub extras: Option<Extras>,
}

/// <https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#reference-animation>
#[derive(Debug, Deserialize)]
pub struct Animation {
    pub channels: Vec<AnimationChannel>,
    pub samplers: Vec<AnimationSampler>,
    pub name: Option<String>,
    pub extensions: Option<Extension>,
    pub extras: Option<Extras>,
}

/// <https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#reference-animation-sampler>
#[derive(Debug, Deserialize)]
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
pub struct AnimationChannel {
    pub sampler: usize,
    pub target: AnimationChannelTarget,
    pub extensions: Option<Extension>,
    pub extras: Option<Extras>,
}

/// <https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#reference-animation-channel-target>
#[derive(Debug, Deserialize)]
pub struct AnimationChannelTarget {
    pub node: Option<usize>,
    pub path: String,
    pub extensions: Option<Extension>,
    pub extras: Option<Extras>,
}

/// <https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#reference-scene>
#[derive(Debug, Deserialize)]
pub struct Scene {
    pub nodes: Vec<usize>,
    pub name: Option<String>,
    pub extensions: Option<Extension>,
    pub extras: Option<Extras>,
}

/// <https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#reference-sampler>
#[derive(Debug, Deserialize)]
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
pub struct Node {
    pub camera: Option<usize>,
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
pub struct Mesh {
    pub primitives: Vec<MeshPrimitive>,
    pub weights: Vec<f32>,
    pub name: Option<String>,
    pub extensions: Option<Extension>,
    pub extras: Option<Extras>,
}

/// <https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#reference-mesh-primitive>
#[derive(Debug, Deserialize)]
pub struct MeshPrimitive {
    pub attributes: JSONValue,
    pub indices: Option<usize>,
    pub material: Option<usize>,
    #[serde(default = "default_mesh_primitive_mode")]
    pub mode: usize,
    pub targets: Vec<JSONValue>,
    pub extensions: Option<Extension>,
    pub extras: Option<Extras>,
}

const fn default_mesh_primitive_mode() -> usize {
    4
}

/// <https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#reference-material>
#[derive(Debug, Deserialize)]
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
pub struct TextureInfo {
    pub index: usize,
    #[serde(default)]
    pub tex_coord: usize,
    pub extensions: Option<Extension>,
    pub extras: Option<Extras>,
}

/// <https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#reference-material-pbrmetallicroughness>
#[derive(Debug, Deserialize)]
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
pub struct Buffer {
    pub uri: Option<String>,
    pub byte_length: usize,
    pub name: Option<String>,
    pub extensions: Option<Extension>,
    pub extras: Option<Extras>,
}

/// <https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#reference-accessor>
#[derive(Debug, Deserialize)]
pub struct Accessor {
    pub buffer_view: Option<usize>,
    #[serde(default)]
    pub byte_offset: usize,
    pub component_type: usize,
    #[serde(default)]
    pub normalized: bool,
    pub count: usize,
    pub r#type: String,
    pub max: Option<usize>,
    pub min: Option<usize>,
    pub sparse: AccessorSparse,
    pub name: Option<String>,
    pub extensions: Option<Extension>,
    pub extras: Option<Extras>,
}

/// <https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#reference-accessor-sparse>
#[derive(Debug, Deserialize)]
pub struct AccessorSparse {
    pub count: usize,
    pub indices: AccessorSparseIndices,
    pub values: AccessorSparseValues,
    pub extensions: Option<Extension>,
    pub extras: Option<Extras>,
}

/// <https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#reference-accessor-sparse-indices>
#[derive(Debug, Deserialize)]
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
    pub generation: Option<String>,
    pub version: String,
    pub min_version: Option<String>,
    pub extensions: Option<Extension>,
    pub extras: Option<Extras>,
}

type Extension = JSONValue;
type Extras = JSONValue;
