use crate::texture;

#[derive(Debug)]
pub struct Sprite {
    pub texture: texture::Id,
    pub texture_rect: Option<texture::Rect>,
}
