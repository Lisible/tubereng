#![warn(clippy::pedantic)]

pub struct EngineConfiguration {
    pub application_title: &'static str,
}

pub struct Engine {
    application_title: &'static str,
}

impl Engine {
    #[must_use]
    pub fn new(configuration: &EngineConfiguration) -> Self {
        Self {
            application_title: configuration.application_title,
        }
    }

    pub fn update(&mut self) {
        println!("update");
    }
    pub fn render(&mut self) {
        println!("render");
    }

    #[must_use]
    pub fn application_title(&self) -> &'static str {
        self.application_title
    }
}
