#![warn(clippy::pedantic)]

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    marker::PhantomData,
};

pub type Result<T> = std::result::Result<T, AssetError>;

#[derive(Debug)]
pub enum AssetError {
    PathCanonicalizationFailed,
}

#[derive(Debug, Clone, Copy)]
pub struct AssetHandle<T> {
    id: usize,
    _marker: PhantomData<T>,
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
        let bytes = FS::read_bytes(asset_path);
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
    fn read_bytes(path: &str) -> Vec<u8>;
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
    fn read_bytes(path: &str) -> Vec<u8> {
        std::fs::read(path).unwrap()
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
        fn read_bytes(_path: &str) -> Vec<u8> {
            vec![]
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
