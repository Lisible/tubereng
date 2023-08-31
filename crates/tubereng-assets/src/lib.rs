#![warn(clippy::pedantic)]

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    hash::Hasher,
    marker::PhantomData,
    path::PathBuf,
};

pub type Result<T> = std::result::Result<T, AssetError>;

#[derive(Debug)]
pub enum AssetError {
    PathCanonicalizationFailed,
    ImageDecodingFailed,
    ReadFailed,
}

#[derive(Debug)]
pub struct AssetHandle<T> {
    id: usize,
    _marker: PhantomData<T>,
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
        self.id.hash(state)
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

pub struct AssetStore<FileSys = FS> {
    assets: HashMap<TypeId, Vec<Box<dyn Any>>>,
    _marker: PhantomData<FileSys>,
}
impl<FS> AssetStore<FS>
where
    FS: FileSystem,
{
    #[must_use]
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
            _marker: PhantomData,
        }
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
        let mut resolved_asset_path = if let Ok(manifest_path) = std::env::var("CARGO_MANIFEST_DIR")
        {
            PathBuf::from(manifest_path)
        } else {
            let mut path = std::env::current_exe().unwrap();
            path.pop();
            path
        };

        resolved_asset_path.push("assets/");
        resolved_asset_path.push(asset_path);
        let bytes = FS::read_bytes(resolved_asset_path.to_str().unwrap())?;
        let asset = A::Loader::load(&bytes)?;
        let assets = self
            .assets
            .entry(TypeId::of::<A>())
            .or_insert_with(Vec::new);
        let asset_id = assets.len();
        assets.push(Box::new(asset));
        Ok(AssetHandle::new(asset_id))
    }

    pub fn get<T: 'static>(&self, handle: AssetHandle<T>) -> Option<&T> {
        Some(
            self.assets[&TypeId::of::<T>()]
                .get(handle.id)?
                .downcast_ref()?,
        )
    }
}

impl<FS> Default for AssetStore<FS>
where
    FS: FileSystem,
{
    fn default() -> Self {
        Self::new()
    }
}

pub trait FileSystem {
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

pub struct FS;
impl FileSystem for FS {
    fn read_bytes(path: &str) -> Result<Vec<u8>> {
        Ok(std::fs::read(path).map_err(|_| AssetError::ReadFailed)?)
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

    pub struct MockFS;
    impl FileSystem for MockFS {
        fn read_bytes(_path: &str) -> std::result::Result<Vec<u8>, AssetError> {
            Ok(vec![])
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
}
