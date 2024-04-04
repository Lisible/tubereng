use crate::Result;

pub mod filesystem;

pub trait VirtualFileSystem {
    /// Reads the content of the file at the given path
    ///
    /// # Errors
    /// An error will be returned if the file cannot be read
    fn read_bytes(&self, path: &str) -> Result<Vec<u8>>;
}
