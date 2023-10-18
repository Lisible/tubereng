#![warn(clippy::pedantic)]

use std::{cell::RefMut, marker::PhantomData, any::{Any, TypeId}};
use tubereng_input::{Input, InputState};

use log::{debug, info};
use tubereng_assets::{AssetStore, FS};
use tubereng_ecs::{
    system::{Into, System, SystemFn},
    Ecs,
};
use tubereng_core::DeltaTime;
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
    should_exit: bool,
}

pub struct ExitRequest;

impl<R> Engine<R>
where
    R: RenderPipeline,
{
    #[must_use]
    pub fn application_title(&self) -> &'static str {
        self.application_title
    }

    /// Initializes the renderer
    ///
    /// # Panics
    ///
    /// Panics if the render pipeline initialization fails
    pub fn initialize_renderer(&mut self, mut renderer: Renderer<R>) {
        let asset_store = self.ecs.resource_mut::<AssetStore>();
        // SAFETY: When the engine is bult by the `EngineBuilder`, an AssetStore resource is created
        // So it should be present anyway
        let mut asset_store = unsafe { asset_store.unwrap_unchecked() };
        renderer.initialize_render_pipeline(&self.render_pipeline_settings, &mut asset_store).unwrap();
        self.renderer = Some(renderer);

        info!("Renderer initialized");
    }

    pub fn run_setup_system(&mut self) {
        self.ecs.run_setup_system();
    }

    pub fn update(&mut self, delta_time: f32) { 
        self.update_delta_time_resource(delta_time);
        self.ecs.run_systems();
        self.ecs.execute_pending_commands();

        let pending_events = self.ecs.event_queue_mut().drain(..).collect::<Vec<_>>();
        for _ in Self::event_iter::<ExitRequest>(&pending_events) {
            self.exit();
        }
    }

    fn update_delta_time_resource(&mut self, delta_time: f32) {
        let mut delta_time_resource = self.ecs.resource_mut::<DeltaTime>().expect("No DeltaTime resource found in Ecs");
        delta_time_resource.0 = delta_time;
    }

    // TODO: Change this
    fn event_iter<E>(pending_events: &[Box<dyn Any>]) -> impl Iterator<Item = &E> where E: 'static {
        pending_events
            .iter()
            .filter(|e| (***e).type_id() == TypeId::of::<E>())
            .map(|e| 
                // SAFETY: We filtered items with the type id of E
                // so they can only be E instances
                unsafe { e.downcast_ref::<E>().unwrap_unchecked() })
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

    pub fn should_exit(&self) -> bool {
        self.should_exit
    }

    pub fn exit(&mut self) {
        self.should_exit = true;
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
        ecs.insert_resource(DeltaTime(0.0));

        Engine {
            application_title: self.application_title.unwrap_or("TuberApp"),
            ecs,
            renderer: None,
            render_pipeline_settings: self.render_pipeline_settings,
            should_exit: false,
        }
    }
}

impl Default for EngineBuilder {
    fn default() -> Self {
        Self::new()
    }
}
