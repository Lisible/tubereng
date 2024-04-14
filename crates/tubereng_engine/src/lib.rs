#![warn(clippy::pedantic)]

use std::sync::Arc;
use tubereng_asset::AssetLoader;
use tubereng_asset::AssetStore;

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use tubereng_core::DeltaTime;
use tubereng_image::ImageLoader;
use tubereng_input::{Input, InputState};

use tubereng_ecs::{
    system::{self, System},
    Ecs,
};
use tubereng_renderer::texture;

#[cfg(not(target_arch = "wasm32"))]
use tubereng_asset::vfs::filesystem::FileSystem;
#[cfg(target_arch = "wasm32")]
use {
    include_dir::{include_dir, Dir},
    tubereng_asset::vfs::web::Web,
};

#[cfg(target_arch = "wasm32")]
static ASSETS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/");

pub struct Engine {
    application_title: &'static str,
    ecs: Ecs,
    init_system: System,
    init_system_ran: bool,
}

impl Engine {
    #[must_use]
    pub fn builder() -> EngineBuilder {
        EngineBuilder::default()
    }

    pub async fn init_graphics<W>(&mut self, window: Arc<W>)
    where
        W: HasWindowHandle + HasDisplayHandle + std::marker::Send + std::marker::Sync,
    {
        // SAFETY: The placeholder image is a valid PNG file that is loaded at compile time
        let placeholder_texture_image = unsafe {
            ImageLoader::load(include_bytes!("../res/placeholder.png")).unwrap_unchecked()
        };
        let placeholder_texture_descriptor = texture::Descriptor {
            data: placeholder_texture_image.data(),
            width: placeholder_texture_image.width(),
            height: placeholder_texture_image.height(),
        };
        tubereng_renderer::renderer_init(&mut self.ecs, window, &placeholder_texture_descriptor)
            .await;
    }

    /// Updates the state of the engine
    pub fn update(&mut self, delta_time: f32) {
        self.ecs.insert_resource(DeltaTime(delta_time));
        if !self.init_system_ran {
            self.ecs.run_single_run_system(&self.init_system);
            self.init_system_ran = true;
        }
        self.ecs.run_systems();
    }

    /// Handles the input
    ///
    /// # Panics
    ///
    /// Will panic if
    /// - the ``InputState`` is missing from the engine resources
    /// - the ``gui::Context`` is missing from the engine resources
    pub fn on_input(&mut self, input: Input) {
        let mut input_state = self
            .ecs
            .resource_mut::<InputState>()
            .expect("InputState should be present in the engine's resources");
        input_state.on_input(&input);
    }

    #[must_use]
    pub fn application_title(&self) -> &'static str {
        self.application_title
    }
}

pub struct EngineBuilder {
    application_title: &'static str,
    init_system: Option<system::System>,
}

impl EngineBuilder {
    pub fn with_application_title(&mut self, application_title: &'static str) -> &mut Self {
        self.application_title = application_title;
        self
    }

    pub fn with_init_system<F, A>(&mut self, init_system: F) -> &mut Self
    where
        F: 'static + system::Into<A>,
    {
        self.init_system = Some(init_system.into_system());
        self
    }

    pub fn build(&mut self) -> Engine {
        let mut ecs = Ecs::new();
        ecs.insert_resource(InputState::new());
        #[cfg(not(target_arch = "wasm32"))]
        ecs.insert_resource(AssetStore::new(FileSystem));
        #[cfg(target_arch = "wasm32")]
        ecs.insert_resource(AssetStore::new(Web::new(&ASSETS)));

        let init_system = self
            .init_system
            .take()
            .unwrap_or(system::Into::<()>::into_system(system::Noop));
        Engine {
            application_title: self.application_title,
            ecs,
            init_system,
            init_system_ran: false,
        }
    }
}

impl Default for EngineBuilder {
    fn default() -> Self {
        Self {
            application_title: "Tuber application",
            init_system: None,
        }
    }
}
