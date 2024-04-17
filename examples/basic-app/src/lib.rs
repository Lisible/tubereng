#![warn(clippy::all)]

use log::warn;
use tubereng::{
    asset::AssetStore,
    core::{DeltaTime, Transform},
    ecs::{
        commands::CommandQueue,
        system::{stages, Res, ResMut, Q},
    },
    engine::Engine,
    image::Image,
    input::{keyboard::Key, InputState},
    math::vector::{Vector2f, Vector3f},
    renderer::texture,
    renderer::{
        camera,
        sprite::{AnimatedSprite, AnimationState, Sprite},
        texture::Rect,
        GraphicsState,
    },
    winit::WinitTuberRunner,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[derive(Debug, Default)]
struct Player {
    acceleration: Vector2f,
    velocity: Vector2f,
}
#[derive(Debug)]
struct Enemy;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let engine = Engine::builder()
        .with_application_title("basic-app")
        .with_init_system(init)
        .build();
    WinitTuberRunner::run(engine).await.unwrap();
}

fn init(queue: &CommandQueue, asset_store: ResMut<AssetStore>, mut gfx: ResMut<GraphicsState>) {
    let image = asset_store
        .load_without_storing::<Image>("texture_atlas.png")
        .unwrap();

    let texture_id = gfx.load_texture(&texture::Descriptor {
        data: image.data(),
        width: image.width(),
        height: image.height(),
    });

    queue.insert((camera::D2::new(800.0, 600.0), camera::Active));

    queue.insert((
        Player::default(),
        Transform {
            translation: Vector3f::new(0.0, 600.0 / 4.0 - 21.0, 0.0),
            scale: Vector3f::new(4.0, 4.0, 4.0),
            ..Default::default()
        },
        AnimatedSprite {
            texture_atlas: texture_id,
            animation: AnimationState {
                animations: vec![vec![
                    Rect::new(16.0, 0.0, 16.0, 16.0),
                    Rect::new(32.0, 0.0, 16.0, 16.0),
                ]],
                current_animation: 0,
                current_frame: 0,
                secs_per_frame: 0.5,
                ticks: 0.0,
            },
        },
    ));

    for i in 0..13 {
        queue.insert((
            Transform {
                translation: Vector3f::new(i as f32 * 16.0, 600.0 / 4.0 - 16.0, 0.0),
                scale: Vector3f::new(4.0, 4.0, 4.0),
                ..Default::default()
            },
            Sprite {
                texture: texture_id,
                texture_rect: Some(Rect::new(0.0, 0.0, 16.0, 16.0)),
            },
        ));
    }

    queue.register_system(&stages::Update, move_player_system);
}

#[derive(Debug)]
pub struct Jumping;

fn move_player_system(
    mut query_player: Q<(&mut Player, &mut Transform)>,
    delta_time: Res<DeltaTime>,
    input_state: Res<InputState>,
) {
    const MAX_PLAYER_VELOCITY: f32 = 100.0;
    const FRICTION: f32 = 0.4;
    let Some((mut player, mut transform)) = query_player.first() else {
        return;
    };
    let delta_time = delta_time.0;

    if input_state.keyboard.is_key_down(Key::D) {
        player.acceleration.x = 1.0;
    } else if input_state.keyboard.is_key_down(Key::A) {
        player.acceleration.x = -1.0;
    } else {
        player.acceleration.x = 0.0;
    }

    if player.velocity.x > 0.0 && player.acceleration.x.abs() < 0.01 {
        player.velocity.x -= FRICTION;
    } else if player.velocity.x < 0.0 && player.acceleration.x.abs() < 0.01 {
        player.velocity.x += FRICTION;
    }

    player.velocity.x += player.acceleration.x;
    if player.velocity.x > MAX_PLAYER_VELOCITY {
        player.velocity.x = MAX_PLAYER_VELOCITY;
    } else if player.velocity.x < -MAX_PLAYER_VELOCITY {
        player.velocity.x = -MAX_PLAYER_VELOCITY;
    }

    player.velocity.y += player.acceleration.y;
    if player.velocity.y > MAX_PLAYER_VELOCITY {
        player.velocity.y = MAX_PLAYER_VELOCITY;
    } else if player.velocity.y < -MAX_PLAYER_VELOCITY {
        player.velocity.y = -MAX_PLAYER_VELOCITY;
    }

    if transform.translation.y > 600.0 / 4.0 - 21.0 {
        transform.translation.y = 600.0 / 4.0 - 21.0;
        player.acceleration.y = 0.0;
        player.velocity.y = 0.0;
    }

    transform.translation.x += player.velocity.x * delta_time;
    transform.translation.y += player.velocity.y * delta_time;
}
