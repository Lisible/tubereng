#![warn(clippy::pedantic)]

use std::{any::Any, hash::Hasher, marker::PhantomData, path::PathBuf};

pub type Result<T> = std::result::Result<T, AssetError>;

#[derive(Debug)]
pub enum AssetError {
    PathCanonicalizationFailed,
    ImageDecodingFailed,
    ReadFailed,
    AssetPathIsInvalidUTF8,
    ExecutablePathAcquisitionFailed(std::io::Error),
    RonAssetParsingFailed(ron::error::SpannedError),
}

#[derive(Debug)]
pub struct AssetHandle<T> {
    id: usize,
    _marker: PhantomData<T>,
}

impl<T> AssetHandle<T> {
    #[must_use]
    pub fn id(&self) -> usize {
        self.id
    }
}

impl<T> Clone for AssetHandle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for AssetHandle<T> {}

impl<T> PartialEq for AssetHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for AssetHandle<T> {}

impl<T> std::hash::Hash for AssetHandle<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T: 'static> AssetHandle<T> {
    #[must_use]
    fn new(id: usize) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }
}

pub struct AssetStore<FileSys = FS>
where
    FileSys: Send + Sync,
{
    assets: Vec<Box<dyn Any + Send + Sync>>,
    _marker: PhantomData<FileSys>,
}
impl<FS> AssetStore<FS>
where
    FS: FileSystem + Send + Sync,
{
    #[must_use]
    pub fn new() -> Self {
        Self {
            assets: vec![],
            _marker: PhantomData,
        }
    }

    /// Loads an asset using an asset path and returns the asset without storing it
    ///
    /// # Errors
    ///
    /// This function will return an error if the canonicalization of the path fails,
    /// or if the asset cannot be loaded.
    pub fn load_without_storing<A>(&self, asset_path: &str) -> Result<A>
    where
        A: 'static + Asset,
    {
        let mut resolved_asset_path = if let Ok(manifest_path) = std::env::var("CARGO_MANIFEST_DIR")
        {
            PathBuf::from(manifest_path)
        } else {
            let mut path =
                std::env::current_exe().map_err(AssetError::ExecutablePathAcquisitionFailed)?;
            path.pop();
            path
        };

        resolved_asset_path.push("assets/");
        resolved_asset_path.push(asset_path);
        let bytes = FS::read_bytes(
            resolved_asset_path
                .to_str()
                .ok_or(AssetError::AssetPathIsInvalidUTF8)?,
        )?;
        A::Loader::load(&bytes)
    }

    /// Loads an asset using an asset path
    ///
    /// # Errors
    ///
    /// This function will return an error if the canonicalization of the path fails,
    /// or if the asset cannot be loaded.
    pub fn load<A>(&mut self, asset_path: &str) -> Result<AssetHandle<A>>
    where
        A: 'static + Asset + Send + Sync,
    {
        Ok(self.store(self.load_without_storing(asset_path)?))
    }

    pub fn store<A>(&mut self, asset: A) -> AssetHandle<A>
    where
        A: 'static + Asset + Send + Sync,
    {
        let asset_id = self.assets.len();
        self.assets.push(Box::new(asset));
        AssetHandle::new(asset_id)
    }

    #[must_use]
    pub fn get<T: 'static>(&self, handle: AssetHandle<T>) -> Option<&T> {
        self.assets.get(handle.id)?.downcast_ref()
    }
}

impl<FS> Default for AssetStore<FS>
where
    FS: FileSystem + Send + Sync,
{
    fn default() -> Self {
        Self::new()
    }
}

pub trait FileSystem {
    /// Reads the content of the file at the given path
    ///
    /// # Errors
    /// An error will be returned if the file cannot be read
    fn read_bytes(path: &str) -> Result<Vec<u8>>;
}

pub trait Asset: Sized {
    type Loader: AssetLoader<Self>;
}

pub trait AssetLoader<T> {
    /// Loads an asset
    ///
    /// # Errors
    ///
    /// This function will return an error if the the asset cannot be loaded
    fn load(file_content: &[u8]) -> Result<T>;
}

pub trait RonAsset {}
impl<T> Asset for T
where
    T: RonAsset + for<'a> serde::Deserialize<'a> + serde::Serialize,
{
    type Loader = RonAssetLoader;
}

pub struct RonAssetLoader;
impl<T> AssetLoader<T> for RonAssetLoader
where
    T: RonAsset + for<'a> serde::Deserialize<'a> + serde::Serialize,
{
    fn load(file_content: &[u8]) -> Result<T> {
        ron::de::from_bytes(file_content).map_err(AssetError::RonAssetParsingFailed)
    }
}

pub struct FS;
impl FileSystem for FS {
    fn read_bytes(path: &str) -> Result<Vec<u8>> {
        std::fs::read(path).map_err(|_| AssetError::ReadFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub struct Text(String);
    impl Asset for Text {
        type Loader = TextAssetLoader;
    }

    pub struct TextAssetLoader;
    impl AssetLoader<Text> for TextAssetLoader {
        fn load(_file_content: &[u8]) -> Result<Text> {
            Ok(Text("cheh".into()))
        }
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    pub struct Material {
        texture: String,
    }

    impl RonAsset for Material {}

    pub struct MockFS;
    impl FileSystem for MockFS {
        fn read_bytes(path: &str) -> std::result::Result<Vec<u8>, AssetError> {
            if path.contains("material.ron") {
                let asset_str = "Material(
                        texture: \"texture.png\",
                      )
                    ";
                Ok(asset_str.as_bytes().to_vec())
            } else {
                Ok(vec![])
            }
        }
    }

    #[test]
    fn asset_store_new() -> Result<()> {
        let mut asset_store = AssetStore::<MockFS>::new();
        let asset_handle = asset_store.load::<Text>("test.txt")?;
        assert_eq!(asset_handle.id, 0);
        Ok(())
    }

    #[test]
    fn asset_store_get() -> Result<()> {
        let mut asset_store = AssetStore::<MockFS>::new();
        let asset_handle = asset_store.load::<Text>("test.txt")?;
        let asset = asset_store.get(asset_handle).unwrap();
        assert_eq!(&asset.0, "cheh");
        Ok(())
    }

    #[test]
    fn asset_store_load_ron() -> Result<()> {
        let mut asset_store = AssetStore::<MockFS>::new();
        let asset_handle = asset_store.load::<Material>("material.ron")?;
        let asset = asset_store.get(asset_handle).unwrap();
        assert_eq!(&asset.texture, "texture.png");
        Ok(())
    }
}
