#![warn(clippy::pedantic)]

use std::sync::Arc;

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use tubereng_input::{Input, InputState};
use tubereng_renderer::GraphicsState;

use tubereng_ecs::{
    system::{self},
    Ecs,
};

pub struct Engine {
    application_title: &'static str,
    ecs: Ecs,
}

impl Engine {
    #[must_use]
    pub fn builder() -> EngineBuilder {
        EngineBuilder::new()
    }

    pub async fn init_graphics<W>(&mut self, window: Arc<W>)
    where
        W: HasWindowHandle + HasDisplayHandle + std::marker::Send + std::marker::Sync,
    {
        self.ecs.insert_resource(GraphicsState::new(window).await);
        self.ecs.insert_resource(tubereng_gui::Context);
        self.ecs
            .register_system::<system::stages::Update, _, _>(tubereng_renderer::update_clear_color);
        self.ecs.register_system::<system::stages::Render, _, _>(
            tubereng_renderer::render_frame_system,
        );
    }

    pub fn update(&mut self) {
        self.ecs.run_systems();
    }

    /// Handles the input
    ///
    /// # Panics
    ///
    /// Will panic if the ``InputState`` is missing from the engine resources
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
    #[must_use]
    pub fn new() -> Self {
        Self {
            application_title: "Tuber application",
            init_system: None,
        }
    }

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
        ecs.run_single_run_system(
            &self
                .init_system
                .take()
                .unwrap_or(system::Into::<()>::into_system(system::Noop)),
        );
        Engine {
            application_title: self.application_title,
            ecs,
        }
    }
}

impl Default for EngineBuilder {
    fn default() -> Self {
        Self::new()
    }
}
