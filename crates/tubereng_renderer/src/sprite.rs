use std::time::Instant;

use crate::texture;

#[derive(Debug)]
pub struct Sprite {
    pub texture: texture::Id,
    pub texture_rect: Option<texture::Rect>,
}

#[derive(Debug)]
pub struct AnimationState {
    pub animations: Vec<Vec<texture::Rect>>,
    pub current_animation: usize,
    pub current_frame: usize,
    pub framerate_millis: usize,
    pub timer: Instant,
}

impl Default for AnimationState {
    fn default() -> Self {
        Self {
            animations: vec![],
            current_animation: 0,
            current_frame: 0,
            framerate_millis: 500,
            timer: Instant::now(),
        }
    }
}

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct AnimatedSprite {
    pub texture_atlas: texture::Id,
    pub animation: AnimationState,
}
