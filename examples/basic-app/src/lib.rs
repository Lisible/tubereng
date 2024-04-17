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
    math::vector::Vector3f,
    renderer::texture,
    renderer::{camera, sprite::Sprite, GraphicsState},
    winit::WinitTuberRunner,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[derive(Debug)]
struct Player;
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
        .load_without_storing::<Image>("texture.png")
        .unwrap();

    let texture_id = gfx.load_texture(&texture::Descriptor {
        data: image.data(),
        width: image.width(),
        height: image.height(),
    });

    queue.insert((camera::D2::new(800.0, 600.0), camera::Active));

    let player = queue.insert((
        Player,
        Transform {
            translation: Vector3f::new(0.0, 0.0, 0.0),
            scale: Vector3f::new(1.0, 1.0, 1.0),
            ..Default::default()
        },
        Sprite {
            texture: texture_id,
            texture_rect: None,
        },
    ));

    let other_entity = queue.insert((
        Transform {
            translation: Vector3f::new(200.0, 200.0, 0.0),
            scale: Vector3f::new(1.0, 1.0, 1.0),
            ..Default::default()
        },
        Sprite {
            texture: texture_id,
            texture_rect: None,
        },
    ));

    queue.insert_relationship::<ChildOf>(other_entity, player);
    queue.register_system(&stages::Update, move_player);
}

fn move_player(
    input: Res<InputState>,
    mut query_player: Q<(&Player, &mut Transform)>,
    dt: Res<DeltaTime>,
) {
    let (_, mut transform) = query_player
        .first()
        .expect("A player should be present in the scene");

    let velocity = 200.0 * dt.0;
    if input.keyboard.is_key_down(Key::W) {
        transform.translation.y -= velocity;
    } else if input.keyboard.is_key_down(Key::S) {
        transform.translation.y += velocity;
    }

    if input.keyboard.is_key_down(Key::A) {
        transform.translation.x -= velocity;
    } else if input.keyboard.is_key_down(Key::D) {
        transform.translation.x += velocity;
    }
}
