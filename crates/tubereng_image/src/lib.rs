#![warn(clippy::pedantic)]

use std::io::Cursor;

use tubereng_asset::{Asset, AssetError, AssetLoader};

#[derive(Debug)]
pub enum ImageError {
    ReaderError(std::io::Error),
    DecodingFailed(image::ImageError),
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    RGBA8,
}

pub struct Image {
    data: Vec<u8>,
    width: u32,
    height: u32,
    format: ImageFormat,
}

impl Image {
    #[must_use]
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    #[must_use]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[must_use]
    pub fn height(&self) -> u32 {
        self.height
    }

    #[must_use]
    pub fn format(&self) -> ImageFormat {
        self.format
    }
}

impl Asset for Image {
    type Loader = ImageLoader;
}

pub struct ImageLoader;
impl AssetLoader<Image> for ImageLoader {
    fn load(file_content: &[u8]) -> tubereng_asset::Result<Image> {
        let cursor = Cursor::new(file_content);
        let image_reader = image::io::Reader::new(cursor);
        let image = image_reader
            .with_guessed_format()
            .map_err(|_| AssetError::ImageDecodingFailed)?
            .decode()
            .map_err(|_| AssetError::ImageDecodingFailed)?;

        let width = image.width();
        let height = image.height();

        Ok(Image {
            data: image.into_rgba8().into_vec(),
            width,
            height,
            format: ImageFormat::RGBA8,
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn load_image() {
        let image_data = include_bytes!("../res/logo.png");
        let image = ImageLoader::load(image_data).unwrap();
        assert_eq!(image.width(), 200);
        assert_eq!(image.height(), 200);
        assert_eq!(image.format(), ImageFormat::RGBA8);
        assert_eq!(
            image.data().len(),
            image.width() as usize * image.height() as usize * 4usize
        );
    }
}
