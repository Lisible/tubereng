#![warn(clippy::pedantic)]

use log::info;
use tubereng_ecs::{
    system::{Into, System, SystemFn},
    Ecs,
};
use tubereng_graphics::{Renderer, WindowSize};

pub struct Engine {
    application_title: &'static str,
    ecs: Ecs,
    renderer: Option<Renderer>,
}

impl Engine {
    #[must_use]
    pub fn application_title(&self) -> &'static str {
        self.application_title
    }

    pub fn initialize_renderer(&mut self, renderer: Renderer) {
        self.renderer = Some(renderer);
        info!("Renderer initialized");
    }

    pub fn run_setup_system(&mut self) {
        self.ecs.run_setup_system();
    }

    pub fn update(&mut self) {
        self.ecs.run_systems();
        self.ecs.execute_pending_commands();
    }

    pub fn render(&mut self) {
        self.renderer
            .as_mut()
            .expect("The renderer is uninitialized")
            .render();
    }

    pub fn resize(&mut self, new_size: WindowSize) {
        self.renderer
            .as_mut()
            .expect("The renderer is uninitialized")
            .resize(new_size);
    }
}

pub struct EngineBuilder {
    application_title: Option<&'static str>,
    setup_system: Option<Box<dyn System>>,
}

impl EngineBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            application_title: None,
            setup_system: None,
        }
    }

    #[must_use]
    pub fn with_application_title(mut self, application_title: &'static str) -> Self {
        self.application_title = Some(application_title);
        self
    }

    #[must_use]
    pub fn with_setup_system<S, M>(mut self, system: S) -> Self
    where
        S: SystemFn<M>,
        M: 'static,
    {
        self.setup_system = Some(Box::new(Into::into(system)));
        self
    }

    #[must_use]
    pub fn build(self) -> Engine {
        let mut ecs = Ecs::new();
        ecs.register_setup_system(self.setup_system.unwrap_or(Box::new(Into::into(|| {}))));

        Engine {
            application_title: self.application_title.unwrap_or("TuberApp"),
            ecs,
            renderer: None,
        }
    }
}

impl Default for EngineBuilder {
    fn default() -> Self {
        Self::new()
    }
}
