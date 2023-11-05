#![warn(clippy::pedantic)]

use std::{marker::PhantomData, any::{Any, TypeId}, time::Instant};
use tubereng_input::{Input, InputState};

use log::{debug, info, trace};
use tubereng_assets::{AssetStore, FS};
use tubereng_ecs::{
    system::{Into, System, SystemFn},
    Ecs, resource::ResourceRefMut,
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

    pub fn initialize_renderer(&mut self, renderer: Renderer<R>) {
        self.renderer = Some(renderer);
        info!("Renderer initialized");
    }

    pub fn run_setup_system(&mut self) {
        self.ecs.run_setup_system();
    }

    pub fn begin_frame(&self) {
        #[cfg(feature = "profiler")]
        puffin::GlobalProfiler::lock().new_frame();
    }

    #[tubereng_profiling_procmacros::function]
    pub fn update(&mut self, delta_time: f32, #[cfg(feature = "egui")] egui_context: egui::Context) {
        trace!("Begin updating...");
        let now = Instant::now();
        self.update_delta_time_resource(delta_time);

        #[cfg(feature = "egui")]
        self.update_egui_context(egui_context);

        self.ecs.run_systems();
        self.ecs.execute_pending_commands();

        let pending_events = self.ecs.event_queue_mut().drain(..).collect::<Vec<_>>();
        for _ in Self::event_iter::<ExitRequest>(pending_events.as_slice()) {
            self.exit();
        }
        trace!("Updating ended, took {:?}", now.elapsed());
    }

    fn update_delta_time_resource(&mut self, delta_time: f32) {
        self.ecs.insert_resource::<DeltaTime>(DeltaTime(delta_time));
    }

        #[cfg(feature = "egui")]
    fn update_egui_context(&mut self, egui_context: egui::Context) {
        self.ecs.insert_resource::<egui::Context>(egui_context);
    }

    // TODO: Change this
    fn event_iter<E>(pending_events: &[Box<dyn Any + Send>]) -> impl Iterator<Item = &E> where E: 'static {
        pending_events
            .iter()
            .filter(|e| (***e).type_id() == TypeId::of::<E>())
            .map(|e| 
                // SAFETY: We filtered items with the type id of E
                // so they can only be E instances
                unsafe { e.downcast_ref::<E>().unwrap_unchecked() })
    }

    fn input_state(&mut self) -> ResourceRefMut<InputState> {
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

    #[tubereng_profiling_procmacros::function]
    pub fn prepare_render(&mut self) {
        let renderer = self
            .renderer
            .as_mut()
            .expect("The renderer is uninitialized");
        trace!("Preparing frame render...");
        let now = Instant::now();
        renderer
            .prepare_render(
                self.ecs.entity_store(),
                self.ecs.relationship_store(),
                &mut self
                    .ecs
                    .resource_mut::<AssetStore>()
                    .expect("AssetStore is not present in the resources"),
            )
            .unwrap();
        trace!("Frame preparation ended, took {:?}", now.elapsed());
    }

    /// # Panics
    /// Might panic if the rendering fails
    #[cfg(not(feature = "egui"))]
    #[tubereng_profiling_procmacros::function]
    pub fn render(&mut self) {
        let renderer = self
            .renderer
            .as_mut()
            .expect("The renderer is uninitialized");
        trace!("Begin frame render...");
        renderer.render().unwrap();
        trace!("Frame render ended");
    }

    #[cfg(feature = "egui")]
    #[tubereng_profiling_procmacros::function]
    pub fn render(&mut self, egui_context: egui::Context, egui_output: egui::FullOutput) {
        let renderer = self
            .renderer
            .as_mut()
            .expect("The renderer is uninitialized");
        trace!("Begin frame render...");
        renderer.render(egui_context, egui_output).unwrap();
        trace!("Frame render ended");
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
        S: SystemFn<M> + Send,
        M: 'static + Send,
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

        #[cfg(feature = "profiler")]
        puffin::set_scopes_on(true);
        
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
