use tubereng_ecs::system::Q;

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
    pub ticks_per_frame: usize,
    pub ticks: usize,
}

impl Default for AnimationState {
    fn default() -> Self {
        Self {
            animations: vec![],
            current_animation: 0,
            current_frame: 0,
            ticks_per_frame: 1,
            ticks: 0,
        }
    }
}

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct AnimatedSprite {
    pub texture_atlas: texture::Id,
    pub animation: AnimationState,
}

pub fn animate_sprite_system(mut query_animated_sprite: Q<&mut AnimatedSprite>) {
    let now = 0;
    for sprite in query_animated_sprite.iter() {
        if sprite.animation.ticks - now > sprite.animation.ticks_per_frame {
            let animation_frame_count =
                sprite.animation.animations[sprite.animation.current_animation].len();
            sprite.animation.ticks = now;
            sprite.animation.current_frame =
                (sprite.animation.current_frame + 1) % animation_frame_count;
        }
    }
}
