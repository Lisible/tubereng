use tubereng_assets::{AssetHandle, AssetStore};
use tubereng_core::Transform;
use tubereng_ecs::entity::EntityBundle;
use tubereng_ecs::relationship::ChildOf;
use tubereng_gltf::Gltf;
use tubereng_graphics::geometry::MeshDescription;
use tubereng_graphics::material::TextureSource;
use tubereng_graphics::{geometry::MeshAsset, material::MaterialAsset};

pub fn load_assets_for_gltf(
    asset_store: &mut AssetStore,
    gltf: AssetHandle<Gltf>,
) -> (Vec<AssetHandle<MeshAsset>>, Vec<AssetHandle<MaterialAsset>>) {
    let meshes = extract_mesh_assets_from_gltf(asset_store, gltf);
    let materials = extract_material_assets_from_gltf(asset_store, gltf);
    let mesh_handles = store_mesh_assets_into_asset_store(meshes, asset_store);
    let material_handles = store_material_assets_into_asset_store(materials, asset_store);
    (mesh_handles, material_handles)
}

pub fn entity_bundle_for_gltf(
    asset_store: &AssetStore,
    gltf: AssetHandle<Gltf>,
    mesh_handles: &[AssetHandle<MeshAsset>],
    material_handles: &[AssetHandle<MaterialAsset>],
) -> EntityBundle {
    let gltf = asset_store.get(gltf).unwrap();
    let mut entity_ids_for_nodes = vec![];
    let mut bundle = EntityBundle::new();
    for node in gltf.nodes() {
        let node_transform = node.transform().clone();
        let bundle_entity_id = match node.mesh() {
            &Some(mesh) => {
                bundle.add_entity((node_transform, material_handles[mesh], mesh_handles[mesh]))
            }
            None => bundle.add_entity((node_transform,)),
        };
        entity_ids_for_nodes.push(bundle_entity_id);
    }

    for (index, node) in gltf.nodes().iter().enumerate() {
        for &child in node.children() {
            bundle.add_relationship::<ChildOf>(
                entity_ids_for_nodes[child],
                entity_ids_for_nodes[index],
            );
        }
    }

    let root_entity = bundle.add_entity((Transform::default(),));
    for &child in gltf.scenes()[gltf.default_scene()].nodes() {
        bundle.add_relationship::<ChildOf>(entity_ids_for_nodes[child], root_entity);
    }

    bundle
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
