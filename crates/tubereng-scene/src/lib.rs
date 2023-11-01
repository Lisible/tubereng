use tubereng_assets::{AssetHandle, AssetStore};
use tubereng_core::Transform;
use tubereng_ecs::{commands::CommandBuffer, entity::EntityId, relationship::ChildOf};
use tubereng_gltf::Gltf;
use tubereng_graphics::geometry::MeshDescription;
use tubereng_graphics::material::TextureSource;
use tubereng_graphics::{geometry::MeshAsset, material::MaterialAsset};

pub fn insert_gltf_to_scene(
    command_buffer: &CommandBuffer,
    asset_store: &mut AssetStore,
    gltf: AssetHandle<Gltf>,
) -> EntityId {
    let meshes = extract_mesh_assets_from_gltf(asset_store, gltf);
    let materials = extract_material_assets_from_gltf(asset_store, gltf);
    let mesh_handles = store_mesh_assets_into_asset_store(meshes, asset_store);
    let material_handles = store_material_assets_into_asset_store(materials, asset_store);
    create_gltf_scene_entities(
        asset_store,
        gltf,
        command_buffer,
        material_handles,
        mesh_handles,
    )
}

fn create_gltf_scene_entities(
    asset_store: &mut AssetStore,
    gltf: AssetHandle<Gltf>,
    command_buffer: &CommandBuffer,
    material_handles: Vec<AssetHandle<MaterialAsset>>,
    mesh_handles: Vec<AssetHandle<MeshAsset>>,
) -> EntityId {
    let gltf = asset_store.get(gltf).unwrap();
    let mut entity_ids_for_nodes = vec![];
    for node in gltf.nodes() {
        let entity_id = command_buffer.insert(());
        command_buffer.add_component(entity_id, node.transform().clone());
        if let &Some(mesh) = node.mesh() {
            command_buffer.add_component(entity_id, material_handles[mesh]);
            command_buffer.add_component(entity_id, mesh_handles[mesh]);
        }
        entity_ids_for_nodes.push(entity_id);
    }
    for (index, node) in gltf.nodes().iter().enumerate() {
        for &child in node.children() {
            command_buffer.insert_relationship::<ChildOf>(
                entity_ids_for_nodes[child],
                entity_ids_for_nodes[index],
            );
        }
    }
    let root_entity = command_buffer.insert((Transform::default(),));
    for &child in gltf.scenes()[gltf.default_scene()].nodes() {
        command_buffer.insert_relationship::<ChildOf>(entity_ids_for_nodes[child], root_entity);
    }
    root_entity
}

fn store_mesh_assets_into_asset_store(
    meshes: Vec<MeshAsset>,
    asset_store: &mut AssetStore,
) -> Vec<AssetHandle<MeshAsset>> {
    let mut mesh_handles = vec![];
    for mesh in meshes {
        mesh_handles.push(asset_store.store(mesh));
    }
    mesh_handles
}
fn store_material_assets_into_asset_store(
    materials: Vec<MaterialAsset>,
    asset_store: &mut AssetStore,
) -> Vec<AssetHandle<MaterialAsset>> {
    let mut material_handles = vec![];
    for material in materials {
        material_handles.push(asset_store.store(material));
    }
    material_handles
}

fn extract_mesh_assets_from_gltf(
    asset_store: &mut AssetStore,
    gltf: AssetHandle<Gltf>,
) -> Vec<MeshAsset> {
    let gltf = asset_store.get(gltf).unwrap();
    let mut meshes = vec![];
    for mesh in gltf.meshes() {
        meshes.push(MeshAsset {
            mesh_description: MeshDescription {
                vertices: mesh.vertices().to_vec(),
                indices: if mesh.indices().is_empty() {
                    None
                } else {
                    Some(mesh.indices().to_vec())
                },
            },
        });
    }
    meshes
}

fn extract_material_assets_from_gltf(
    asset_store: &mut AssetStore,
    gltf: AssetHandle<Gltf>,
) -> Vec<MaterialAsset> {
    let gltf = asset_store.get(gltf).unwrap();
    let mut materials = vec![];
    for mesh in gltf.meshes() {
        materials.push(MaterialAsset {
            texture: TextureSource::Data(mesh.texture().into()),
        });
    }
    materials
}
