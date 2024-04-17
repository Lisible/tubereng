use tubereng_core::DeltaTime;
use tubereng_ecs::system::{Res, Q};

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
    pub secs_per_frame: f32,
    pub ticks: f32,
}

impl Default for AnimationState {
    fn default() -> Self {
        Self {
            animations: vec![],
            current_animation: 0,
            current_frame: 0,
            secs_per_frame: 1.0,
            ticks: 0.0,
        }
    }
}

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct AnimatedSprite {
    pub texture_atlas: texture::Id,
    pub animation: AnimationState,
}

pub fn animate_sprite_system(
    delta_time: Res<DeltaTime>,
    mut query_animated_sprite: Q<&mut AnimatedSprite>,
) {
    let now = delta_time.0;
    for mut sprite in query_animated_sprite.iter() {
        sprite.animation.ticks += now;
        if sprite.animation.ticks > sprite.animation.secs_per_frame {
            let animation_frame_count =
                sprite.animation.animations[sprite.animation.current_animation].len();
            sprite.animation.ticks -= sprite.animation.secs_per_frame;
            sprite.animation.current_frame =
                (sprite.animation.current_frame + 1) % animation_frame_count;
        }
    }

    std::mem::drop(delta_time);
}
