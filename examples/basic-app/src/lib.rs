#![warn(clippy::all)]

use log::warn;
use tubereng::{
    asset::AssetStore,
    core::{DeltaTime, Transform},
    ecs::{
        commands::CommandQueue,
        relationship::ChildOf,
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

#[cfg(not(target_arch = "wasm32"))]
use tubereng::asset::vfs::filesystem::FileSystem;
#[cfg(target_arch = "wasm32")]
use {
    include_dir::{include_dir, Dir},
    tubereng::asset::vfs::web::Web,
};

#[cfg(target_arch = "wasm32")]
static ASSETS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/assets");

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

    #[cfg(target_arch = "wasm32")]
    let vfs = Web::new(&ASSETS);
    #[cfg(not(target_arch = "wasm32"))]
    let vfs = FileSystem;

    let engine = Engine::builder()
        .with_application_title("basic-app")
        .with_init_system(init)
        .build(vfs);
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

    let camera = queue.insert((
        camera::D2::new(800.0, 600.0),
        camera::Active,
        Transform {
            translation: Vector3f::new(-400.0, -300.0, 0.0),
            scale: Vector3f::uniform(0.8),
            ..Default::default()
        },
    ));

    queue.insert((
        Transform {
            translation: Vector3f::new(0.0, 0.0, -10.0),
            scale: Vector3f::uniform(12.5),
            ..Default::default()
        },
        Sprite {
            texture: texture_id,
            texture_rect: Some(Rect::new(48.0, 0.0, 64.0, 48.0)),
        },
    ));

    let player = queue.insert((
        Player::default(),
        Grounded,
        Transform {
            translation: Vector3f::new(0.0, 600.0 - 85.0, 0.0),
            ..Default::default()
        },
    ));

    let player_sprite = queue.insert((
        Transform {
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

    queue.insert_relationship::<ChildOf>(player_sprite, player);
    queue.insert_relationship::<ChildOf>(camera, player);

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

    queue.register_system(&stages::Update, move_player_grounded_system);
    queue.register_system(&stages::Update, move_player_jumping_system);
}

#[derive(Debug)]
pub struct Jumping;
#[derive(Debug)]
pub struct Grounded;

fn move_player_grounded_system(
    queue: &CommandQueue,
    mut query_player: Q<(&mut Player, &mut Transform, &Grounded)>,
    delta_time: Res<DeltaTime>,
    input_state: Res<InputState>,
) {
    const MAX_PLAYER_VELOCITY_X: f32 = 200.0;
    const MAX_PLAYER_VELOCITY_Y: f32 = 100.0;
    const FRICTION: f32 = 1.0;
    let Some((player_id, (mut player, mut transform, _))) = query_player.first_with_id() else {
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

    if input_state.keyboard.is_key_down(Key::W) {
        player.acceleration.y = -0.2;
        queue.remove_component::<Grounded>(player_id);
        queue.insert_component(player_id, Jumping);
        queue.insert_component(player_id, MaxJumpHeightReached(false));
    } else {
        player.acceleration.y = 0.0;
    }

    if player.velocity.x > 0.0 && player.acceleration.x.abs() < 0.01 {
        player.velocity.x -= FRICTION;
    } else if player.velocity.x < 0.0 && player.acceleration.x.abs() < 0.01 {
        player.velocity.x += FRICTION;
    }

    player.velocity.x += player.acceleration.x;
    if player.velocity.x > MAX_PLAYER_VELOCITY_X {
        player.velocity.x = MAX_PLAYER_VELOCITY_X;
    } else if player.velocity.x < -MAX_PLAYER_VELOCITY_X {
        player.velocity.x = -MAX_PLAYER_VELOCITY_X;
    }

    player.velocity.y += player.acceleration.y;
    if player.velocity.y > MAX_PLAYER_VELOCITY_Y {
        player.velocity.y = MAX_PLAYER_VELOCITY_Y;
    } else if player.velocity.y < -MAX_PLAYER_VELOCITY_Y {
        player.velocity.y = -MAX_PLAYER_VELOCITY_Y;
    }

    transform.translation.x += player.velocity.x * delta_time;
    transform.translation.y += player.velocity.y * delta_time;
}

#[derive(Debug)]
struct MaxJumpHeightReached(pub bool);

fn move_player_jumping_system(
    queue: &CommandQueue,
    mut query_player: Q<(
        &mut Player,
        &mut Transform,
        &Jumping,
        &mut MaxJumpHeightReached,
    )>,
    delta_time: Res<DeltaTime>,
    input_state: Res<InputState>,
) {
    const MAX_PLAYER_VELOCITY_X: f32 = 200.0;
    const MAX_PLAYER_VELOCITY_Y: f32 = 200.0;
    const GRAVITY: f32 = 0.015;
    const FRICTION: f32 = 0.1;
    let Some((player_id, (mut player, mut transform, _, mut max_jump_height_reached))) =
        query_player.first_with_id()
    else {
        return;
    };
    let delta_time = delta_time.0;

    if !max_jump_height_reached.0 && input_state.keyboard.is_key_down(Key::W) {
        player.acceleration.y -= 0.02;
        if player.acceleration.y < -1.1 {
            max_jump_height_reached.0 = true;
        }
    }

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
    if player.velocity.x > MAX_PLAYER_VELOCITY_X {
        player.velocity.x = MAX_PLAYER_VELOCITY_X;
    } else if player.velocity.x < -MAX_PLAYER_VELOCITY_X {
        player.velocity.x = -MAX_PLAYER_VELOCITY_X;
    }

    player.acceleration.y += GRAVITY;
    player.velocity.y += player.acceleration.y;
    if player.velocity.y > MAX_PLAYER_VELOCITY_Y {
        player.velocity.y = MAX_PLAYER_VELOCITY_Y;
    } else if player.velocity.y < -MAX_PLAYER_VELOCITY_Y {
        player.velocity.y = -MAX_PLAYER_VELOCITY_Y;
    }

    transform.translation.x += player.velocity.x * delta_time;
    transform.translation.y += player.velocity.y * delta_time;
    if transform.translation.y > 600.0 - 85.0 {
        transform.translation.y = 600.0 - 85.0;
        player.acceleration.y = 0.0;
        player.velocity.y = 0.0;
        queue.remove_component::<Jumping>(player_id);
        queue.remove_component::<MaxJumpHeightReached>(player_id);
        queue.insert_component(player_id, Grounded);
    }
}
