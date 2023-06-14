#![warn(clippy::pedantic)]

use tubereng_ecs::system::System;

pub struct Engine {
    application_title: &'static str,
    setup_system: Box<dyn System>,
}

impl Engine {
    #[must_use]
    pub fn application_title(&self) -> &'static str {
        self.application_title
    }

    pub fn run_setup_system(&mut self) {
        self.setup_system.run();
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
    pub fn with_setup_system<F>(mut self, setup_system: F) -> Self
    where
        F: 'static + System,
    {
        self.setup_system = Some(Box::new(setup_system));
        self
    }

    #[must_use]
    pub fn build(self) -> Engine {
        Engine {
            application_title: self.application_title.unwrap_or("TuberApp"),
            setup_system: self.setup_system.unwrap_or(Box::new(|| ())),
        }
    }
}

impl Default for EngineBuilder {
    fn default() -> Self {
        Self::new()
    }
}
