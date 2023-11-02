use tubereng_assets::{AssetHandle, AssetStore};
use tubereng_core::Transform;
use tubereng_ecs::entity::EntityBundle;
use tubereng_ecs::relationship::ChildOf;
use tubereng_gltf::Gltf;
use tubereng_graphics::geometry::MeshDescription;
use tubereng_graphics::material::TextureSource;
use tubereng_graphics::{geometry::MeshAsset, material::MaterialAsset};

pub struct Scene {
    mesh_handles: Vec<AssetHandle<MeshAsset>>,
    material_handles: Vec<AssetHandle<MaterialAsset>>,
    gltf: Gltf,
}

impl Scene {
    pub fn from_gltf(gltf: Gltf, asset_store: &mut AssetStore) -> Self {
        let meshes = Self::extract_mesh_assets_from_gltf(&gltf);
        let mesh_handles = Self::store_mesh_assets_into_asset_store(meshes, asset_store);
        let materials = Self::extract_material_assets_from_gltf(&gltf);
        let material_handles = Self::store_material_assets_into_asset_store(materials, asset_store);

        Self {
            mesh_handles,
            material_handles,
            gltf,
        }
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

    fn extract_mesh_assets_from_gltf(gltf: &Gltf) -> Vec<MeshAsset> {
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

    fn extract_material_assets_from_gltf(gltf: &Gltf) -> Vec<MaterialAsset> {
        let mut materials = vec![];
        for mesh in gltf.meshes() {
            materials.push(MaterialAsset {
                texture: TextureSource::Data(mesh.texture().into()),
            });
        }
        materials
    }

    pub fn entity_bundle(&self) -> EntityBundle {
        let mut entity_ids_for_nodes = vec![];
        let mut bundle = EntityBundle::new();
        for node in self.gltf.nodes() {
            let node_transform = node.transform().clone();
            let bundle_entity_id = match node.mesh() {
                &Some(mesh) => bundle.add_entity((
                    node_transform,
                    self.material_handles[mesh],
                    self.mesh_handles[mesh],
                )),
                None => bundle.add_entity((node_transform,)),
            };
            entity_ids_for_nodes.push(bundle_entity_id);
        }

        for (index, node) in self.gltf.nodes().iter().enumerate() {
            for &child in node.children() {
                bundle.add_relationship::<ChildOf>(
                    entity_ids_for_nodes[child],
                    entity_ids_for_nodes[index],
                );
            }
        }

        let root_entity = bundle.add_entity((Transform::default(),));
        bundle.set_root(root_entity);
        for &child in self.gltf.scenes()[self.gltf.default_scene()].nodes() {
            bundle.add_relationship::<ChildOf>(entity_ids_for_nodes[child], root_entity);
        }

        bundle
    }
}
