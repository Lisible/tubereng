use include_dir::Dir;

use crate::AssetError;
use crate::Result;
use log::warn;

use super::VirtualFileSystem;

pub struct Web {
    assets: &'static Dir<'static>,
}

impl Web {
    #[must_use]
    pub fn new(assets: &'static Dir<'static>) -> Self {
        warn!("{:?}", assets);
        Self { assets }
    }
}

impl VirtualFileSystem for Web {
    fn read_bytes(&self, path: &str) -> Result<Vec<u8>> {
        warn!("{:?}", path);
        Ok(self
            .assets
            .get_file(path)
            .ok_or(AssetError::ReadFailed)?
            .contents()
            .to_vec())
    }
}
