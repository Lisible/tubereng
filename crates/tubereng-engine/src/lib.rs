#![warn(clippy::pedantic)]

pub struct Engine {
    application_title: &'static str,
}

impl Engine {
    pub fn application_title(&self) -> &'static str {
        self.application_title
    }
}

pub struct EngineBuilder {
    application_title: Option<&'static str>,
}

impl EngineBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            application_title: None,
        }
    }

    pub fn with_application_title(&mut self, application_title: &'static str) -> &mut Self {
        self.application_title = Some(application_title);
        self
    }

    #[must_use]
    pub fn build(&self) -> Engine {
        Engine {
            application_title: self.application_title.unwrap_or("TuberApp"),
        }
    }
}

impl Default for EngineBuilder {
    fn default() -> Self {
        Self::new()
    }
}
