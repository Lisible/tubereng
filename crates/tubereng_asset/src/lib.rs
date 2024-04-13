#![warn(clippy::pedantic)]

use log::warn;
use std::{any::Any, hash::Hasher, marker::PhantomData, path::PathBuf};

use vfs::VirtualFileSystem;

pub mod vfs;
pub type Result<T> = std::result::Result<T, AssetError>;

#[derive(Debug)]
pub enum AssetError {
    PathCanonicalizationFailed,
    ImageDecodingFailed,
    ReadFailed,
    AssetPathIsInvalidUTF8,
    ExecutablePathAcquisitionFailed(std::io::Error),
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

pub struct AssetStore {
    fs: Box<dyn VirtualFileSystem>,
    assets: Vec<Box<dyn Any>>,
}
impl AssetStore {
    #[must_use]
    pub fn new<FS>(fs: FS) -> Self
    where
        FS: VirtualFileSystem + 'static,
    {
        Self {
            fs: Box::new(fs),
            assets: vec![],
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
        let mut resolved_asset_path = PathBuf::new();
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Ok(manifest_path) = std::env::var("CARGO_MANIFEST_DIR") {
                PathBuf::from(manifest_path)
            } else {
                let mut path =
                    std::env::current_exe().map_err(AssetError::ExecutablePathAcquisitionFailed)?;
                path.pop();
                path
            };
            resolved_asset_path.push("assets/");
        }

        resolved_asset_path.push(asset_path);
        let bytes = self.fs.read_bytes(
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
        A: 'static + Asset,
    {
        Ok(self.store(self.load_without_storing(asset_path)?))
    }

    pub fn store<A>(&mut self, asset: A) -> AssetHandle<A>
    where
        A: 'static + Asset,
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

    pub struct MockFS;
    impl VirtualFileSystem for MockFS {
        fn read_bytes(&self, _path: &str) -> std::result::Result<Vec<u8>, AssetError> {
            Ok(vec![])
        }
    }

    #[test]
    fn asset_store_new() -> Result<()> {
        let fs = MockFS;
        let mut asset_store = AssetStore::new(fs);
        let asset_handle = asset_store.load::<Text>("test.txt")?;
        assert_eq!(asset_handle.id, 0);
        Ok(())
    }

    #[test]
    fn asset_store_get() -> Result<()> {
        let fs = MockFS;
        let mut asset_store = AssetStore::new(fs);
        let asset_handle = asset_store.load::<Text>("test.txt")?;
        let asset = asset_store.get(asset_handle).unwrap();
        assert_eq!(&asset.0, "cheh");
        Ok(())
    }
}
