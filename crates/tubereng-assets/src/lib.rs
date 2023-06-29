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

pub struct AssetHandle<'a, T> {
    inner: TypeErasedAssetHandle<'a>,
    _marker: PhantomData<T>,
}

impl<'a, T: 'static> AssetHandle<'a, T> {
    #[must_use]
    fn new(inner: TypeErasedAssetHandle<'a>) -> Self {
        Self {
            inner,
            _marker: PhantomData,
        }
    }

    #[must_use]
    pub fn get(&self) -> &'a T {
        // SAFETY: We know that inner contains a reference to a T
        // Because we knew its type when we created the handle
        unsafe { self.inner.0.downcast_ref().unwrap_unchecked() }
    }
}

pub struct TypeErasedAssetHandle<'a>(&'a dyn Any);

pub struct AssetStore<FileSys = FS> {
    assets: HashMap<TypeId, HashMap<String, Box<dyn Any>>>,
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
            .or_insert_with(HashMap::new);

        assets.insert(asset_path.to_string(), Box::new(asset));
        Ok(AssetHandle::new(TypeErasedAssetHandle(
            // SAFETY: This is safe as we just added it into the assets
            unsafe { assets.get(asset_path).unwrap_unchecked() }.as_ref(),
        )))
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
        let text = asset_handle.get();
        assert_eq!(text.0, String::from("cheh"));
        Ok(())
    }
}
