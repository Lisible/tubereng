#![warn(clippy::pedantic)]

use std::{cell::RefMut, marker::PhantomData};
use tubereng_input::{Input, InputState};

use log::{debug, info};
use tubereng_assets::{AssetStore, FS};
use tubereng_ecs::{
    system::{Into, System, SystemFn},
    Ecs,
};
use tubereng_graphics::{
    pipeline::{default_pipeline::DefaultRenderPipeline, RenderPipeline},
    Renderer, WindowSize,
};

pub struct Engine<R = DefaultRenderPipeline>
where
    R: RenderPipeline,
{
    application_title: &'static str,
    ecs: Ecs,
    renderer: Option<Renderer<R>>,
    render_pipeline_settings: R::RenderPipelineSettings,
}

impl<R> Engine<R>
where
    R: RenderPipeline,
{
    #[must_use]
    pub fn application_title(&self) -> &'static str {
        self.application_title
    }

    pub fn initialize_renderer(&mut self, renderer: Renderer<R>) {
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

    fn input_state(&mut self) -> RefMut<InputState> {
        self.ecs
            .resource_mut::<InputState>()
            .expect("Input state not foudn in resources")
    }

    pub fn clear_last_frame_inputs(&mut self) {
        self.input_state().clear_last_frame_inputs();
    }

    pub fn on_input(&mut self, input: Input) {
        debug!("Handling input: {:?}", input);
        self.input_state().on_input(input);
    }

    /// # Panics
    /// Might panic if the rendering fails
    pub fn render(&mut self) {
        let renderer = self
            .renderer
            .as_mut()
            .expect("The renderer is uninitialized");
        renderer
            .prepare_render(
                self.ecs.entity_store(),
                &mut self
                    .ecs
                    .resource_mut::<AssetStore>()
                    .expect("AssetStore is not present in the resources"),
            )
            .unwrap();
        renderer.render().unwrap();
    }

    pub fn resize(&mut self, new_size: WindowSize) {
        self.renderer
            .as_mut()
            .expect("The renderer is uninitialized")
            .resize(new_size);
    }

    pub fn render_pipeline_settings(&self) -> &R::RenderPipelineSettings {
        &self.render_pipeline_settings
    }
}

pub struct EngineBuilder<R = DefaultRenderPipeline>
where
    R: RenderPipeline,
{
    application_title: Option<&'static str>,
    setup_system: Option<Box<dyn System>>,
    render_pipeline_settings: R::RenderPipelineSettings,
    _marker: PhantomData<R>,
}

impl<R> EngineBuilder<R>
where
    R: RenderPipeline,
{
    #[must_use]
    pub fn new() -> Self {
        Self {
            application_title: None,
            setup_system: None,
            render_pipeline_settings: R::RenderPipelineSettings::default(),
            _marker: PhantomData,
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
    pub fn with_render_pipeline_settings(
        mut self,
        render_pipeline_settings: R::RenderPipelineSettings,
    ) -> Self {
        self.render_pipeline_settings = render_pipeline_settings;
        self
    }

    #[must_use]
    pub fn build(self) -> Engine<R> {
        let mut ecs = Ecs::new();
        ecs.register_setup_system(self.setup_system.unwrap_or(Box::new(Into::into(|| {}))));
        ecs.insert_resource(AssetStore::<FS>::new());
        ecs.insert_resource(InputState::new());

        Engine {
            application_title: self.application_title.unwrap_or("TuberApp"),
            ecs,
            renderer: None,
            render_pipeline_settings: self.render_pipeline_settings,
        }
    }
}

impl Default for EngineBuilder {
    fn default() -> Self {
        Self::new()
    }
}
