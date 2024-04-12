#![warn(clippy::all)]

use log::warn;
use tubereng::{
    asset::AssetStore,
    core::Transform,
    ecs::{
        commands::CommandQueue,
        system::{stages, Res, ResMut, Q},
    },
    engine::Engine,
    image::Image,
    input::{keyboard::Key, InputState},
    math::vector::Vector3f,
    renderer::texture,
    renderer::{material, sprite::Sprite, GraphicsState},
    winit::WinitTuberRunner,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[derive(Debug)]
struct Player;
#[derive(Debug)]
struct Enemy;

#[cfg(not(target_arch = "wasm32"))]
use tubereng::asset::vfs::filesystem::FileSystem;

#[cfg(target_arch = "wasm32")]
use include_dir::{include_dir, *};
#[cfg(target_arch = "wasm32")]
static ASSETS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/");

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

    #[cfg(not(target_arch = "wasm32"))]
    let vfs = FileSystem;
    #[cfg(target_arch = "wasm32")]
    let vfs = {
        use tubereng::asset::vfs::web::Web;
        Web::new(&ASSETS)
    };

    let engine = Engine::builder()
        .with_application_title("basic-app")
        .with_init_system(init)
        .with_vfs(vfs)
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

    let material_id = gfx.load_material(&material::Descriptor {
        base_color: texture_id,
    });

    queue.insert((
        Player,
        Transform {
            translation: Vector3f::new(0.0, 0.0, 0.0),
            scale: Vector3f::new(0.1, 0.1, 0.1),
            ..Default::default()
        },
        Sprite {
            material: Some(material_id),
        },
    ));
    queue.insert((
        Enemy,
        Transform {
            translation: Vector3f::new(-1.0, -1.0, 0.0),
            scale: Vector3f::new(0.1, 0.1, 0.1),
            ..Default::default()
        },
        Sprite::default(),
    ));
    queue.insert((
        Enemy,
        Transform {
            scale: Vector3f::new(0.1, 0.1, 0.1),
            ..Default::default()
        },
        Sprite::default(),
    ));
    queue.insert((
        Enemy,
        Transform {
            scale: Vector3f::new(0.1, 0.1, 0.1),
            ..Default::default()
        },
        Sprite::default(),
    ));

    queue.register_system(&stages::Update, move_player);
}

fn move_player(input: Res<InputState>, mut query_player: Q<(&Player, &mut Transform)>) {
    let (_, transform) = query_player
        .first()
        .expect("A player should be present in the scene");

    if input.keyboard.is_key_down(Key::S) {
        transform.translation.y -= 0.001;
    } else if input.keyboard.is_key_down(Key::W) {
        transform.translation.y += 0.001;
    }

    if input.keyboard.is_key_down(Key::A) {
        transform.translation.x -= 0.001;
    } else if input.keyboard.is_key_down(Key::D) {
        transform.translation.x += 0.001;
    }
}
