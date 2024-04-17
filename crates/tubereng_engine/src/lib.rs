#![warn(clippy::pedantic)]

use std::sync::Arc;
use tubereng_asset::AssetLoader;
use tubereng_asset::AssetStore;
use tubereng_core::TransformCache;
use tubereng_ecs::system::stages;

use tubereng_math::matrix::Identity;
use tubereng_math::matrix::Matrix4f;

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use tubereng_core::DeltaTime;
use tubereng_core::Transform;
use tubereng_ecs::query::DirtyState;
use tubereng_ecs::relationship::ChildOf;

use tubereng_ecs::Storage;
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
        self.ecs.clear_dirty_flags();
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
        ecs.insert_resource(TransformCache::new());
        #[cfg(not(target_arch = "wasm32"))]
        ecs.insert_resource(AssetStore::new(FileSystem));
        #[cfg(target_arch = "wasm32")]
        ecs.insert_resource(AssetStore::new(Web::new(&ASSETS)));
        ecs.register_system(&stages::Render, compute_effective_transforms_system);

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

fn compute_effective_transforms_system(storage: &Storage) {
    let child_of_relationship = storage
        .relationship::<ChildOf>()
        .expect("ChildOf relationship should be present");

    let mut dirty_transform_entities = vec![];
    for (id, dirty) in storage.query::<DirtyState<Transform>>().iter_with_ids() {
        if !dirty {
            continue;
        }

        let mut id = id;
        if let Some(targets) = child_of_relationship.targets(id) {
            if let Some(&parent_id) = targets.iter().next() {
                let dirty = storage.dirty_state::<Transform>(parent_id);
                if dirty {
                    id = parent_id;
                }
            }
        }

        dirty_transform_entities.push(id);
    }

    let mut transform_cache = storage
        .resource_mut::<TransformCache>()
        .expect("A TransformCache resource should be present");
    while let Some(entity_id) = dirty_transform_entities.pop() {
        let parent = match child_of_relationship.targets(entity_id) {
            Some(parents) => parents.iter().next(),
            None => None,
        };

        let parent_matrix = match parent {
            Some(&parent_id) => match storage.component::<Transform>(parent_id) {
                Some(transform) => transform.as_matrix4(),
                None => Matrix4f::identity(),
            },
            None => Matrix4f::identity(),
        };

        let matrix = storage
            .component::<Transform>(entity_id)
            .unwrap()
            .as_matrix4();

        transform_cache.set(entity_id, parent_matrix * matrix);
        if let Some(children) = child_of_relationship.sources(entity_id) {
            dirty_transform_entities.extend(children.iter());
        }
    }
}
