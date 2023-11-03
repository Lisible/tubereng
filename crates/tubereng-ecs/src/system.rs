use std::{marker::PhantomData, ops::DerefMut};

use crate::{
    commands::CommandBuffer,
    entity::EntityStore,
    event::{EventQueue, EventReader, EventWriter},
    query::{Query, Q},
    relationship::RelationshipStore,
    resource::{ResourceRef, ResourceRefMut, Resources},
};

pub struct SystemSet {
    systems: Vec<Box<dyn System + Send + Sync>>,
}

impl SystemSet {
    #[must_use]
    pub fn new() -> Self {
        Self { systems: vec![] }
    }

    pub fn add_system<S, ST, M>(&mut self, system: S)
    where
        S: Into<M, SystemType = ST> + Send + Sync,
        ST: 'static + System + Send + Sync,
    {
        self.systems.push(Box::new(Into::into(system)));
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<Box<dyn System + Send + Sync>> {
        self.systems.iter_mut()
    }
}

impl Default for SystemSet {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ExecutionContext<'a> {
    pub(crate) command_buffer: &'a CommandBuffer,
    pub(crate) entity_store: &'a EntityStore,
    pub(crate) relationship_store: &'a RelationshipStore,
    pub(crate) resources: &'a Resources,
    pub(crate) event_queue: &'a EventQueue,
}

pub trait System: Send {
    fn execute<'a>(&'a mut self, ctx: &'a ExecutionContext<'a>);
}

pub struct Function<F, M>
where
    F: Send + SystemFn<M>,
{
    system_fn: F,
    _marker: PhantomData<M>,
}

impl<F, M> System for Function<F, M>
where
    F: Send + SystemFn<M>,
    M: Send,
{
    fn execute<'a>(&'a mut self, ctx: &'a ExecutionContext<'a>) {
        let parameters = F::Parameter::fetch(ctx);
        self.system_fn.run(parameters);
    }
}

pub trait Into<M> {
    type SystemType: System;
    fn into(system: Self) -> Self::SystemType;
}

impl System for Box<dyn System + Send + Sync> {
    fn execute<'a>(&'a mut self, ctx: &'a ExecutionContext<'a>) {
        self.deref_mut().execute(ctx);
    }
}

impl Into<()> for Box<dyn System + Send + Sync> {
    type SystemType = Self;

    fn into(system: Self) -> Self::SystemType {
        system
    }
}

impl<F, M> Into<M> for F
where
    F: SystemFn<M> + Send,
    M: Send,
{
    type SystemType = Function<F, M>;

    fn into(system: Self) -> Self::SystemType {
        Function {
            system_fn: system,
            _marker: PhantomData,
        }
    }
}

pub trait SystemFn<M>: 'static {
    type Parameter: Parameter;

    fn run<'a>(&'a mut self, parameters: <Self::Parameter as Parameter>::Item<'a>);
}

impl<F> SystemFn<()> for F
where
    F: FnMut() + Send + Sync + 'static,
{
    type Parameter = ();

    fn run(&mut self, parameters: Self::Parameter) {
        let () = parameters;
        (self)();
    }
}

macro_rules! impl_systemfn_for_tuples {
    ($head:ident, $($tail:ident,)*) => {
        #[allow(non_snake_case)]
        impl<F: 'static, $head, $($tail,)*> SystemFn<fn($head, $($tail,)*)> for F
        where
            for<'a> &'a mut F: FnMut($head, $($tail,)*) + FnMut($head::Item<'a>, $($tail::Item<'a>,)*) + Send + Sync,
            $head: Parameter,
            $($tail: Parameter,)*
        {
            type Parameter = ($head, $($tail,)*);

            fn run<'a>(&'a mut self, parameters: <Self::Parameter as Parameter>::Item<'a>) {
                let ($head, $($tail,)*) = parameters;

                #[allow(clippy::items_after_statements)]
                fn call_inner<$head, $($tail,)*>(mut f: impl FnMut($head, $($tail,)*), $head: $head, $($tail: $tail,)*) {
                    f($head, $($tail,)*);
                }

                call_inner(self, $head, $($tail,)*);
            }
        }

        impl_systemfn_for_tuples!($($tail,)*);
    };

    () => {};
}

impl_systemfn_for_tuples!(A, B, C, D, E,);

pub trait Parameter {
    type Item<'a>;
    fn fetch<'a>(execution_context: &'a ExecutionContext<'a>) -> Self::Item<'a>;
}
impl Parameter for () {
    type Item<'a> = ();
    fn fetch<'a>(_execution_context: &'a ExecutionContext<'a>) -> Self::Item<'a> {}
}

macro_rules! impl_parameter_for_tuples {
    ($head:ident, $($tail:ident,)*) => {
        impl<$head, $($tail,)*> Parameter for ($head, $($tail,)*)
        where
            $head: Parameter,
            $($tail: Parameter,)*
        {
            type Item<'a> = ($head::Item<'a>, $($tail::Item<'a>,)*);
            fn fetch<'a>(execution_context: &'a ExecutionContext<'a>) -> Self::Item<'a> {
                ($head::fetch(execution_context), $($tail::fetch(execution_context),)*)
            }
        }

        impl_parameter_for_tuples!($($tail,)*);
    };
    () => {};
}

impl_parameter_for_tuples!(A, B, C, D, E,);

impl Parameter for &CommandBuffer {
    type Item<'a> = &'a CommandBuffer;
    fn fetch<'a>(execution_context: &'a ExecutionContext<'a>) -> Self::Item<'a> {
        execution_context.command_buffer
    }
}

pub struct Res<'a, T>(pub ResourceRef<'a, T>);
impl<T> Parameter for Res<'_, T>
where
    T: 'static + Send,
{
    type Item<'a> = Res<'a, T>;
    fn fetch<'a>(execution_context: &'a ExecutionContext<'a>) -> Self::Item<'a> {
        Res(execution_context
            .resources
            .resource::<T>()
            .expect("Resource not found"))
    }
}

pub struct ResMut<'a, T>(pub ResourceRefMut<'a, T>);
impl<T> Parameter for ResMut<'_, T>
where
    T: 'static + Send,
{
    type Item<'a> = ResMut<'a, T>;
    fn fetch<'c>(execution_context: &'c ExecutionContext<'c>) -> Self::Item<'c> {
        ResMut(
            execution_context
                .resources
                .resource_mut::<T>()
                .expect("Resource not found"),
        )
    }
}

impl<'q, QD> Parameter for Q<'q, QD>
where
    QD: for<'a> Query<'a>,
{
    type Item<'a> = Q<'a, QD>;

    fn fetch<'a>(execution_context: &'a ExecutionContext<'a>) -> Self::Item<'a> {
        Q::new(
            execution_context.entity_store,
            execution_context.relationship_store,
        )
    }
}

impl<'q, E> Parameter for EventWriter<'q, E>
where
    E: 'static + Send,
{
    type Item<'a> = EventWriter<'a, E>;

    fn fetch<'a>(execution_context: &'a ExecutionContext<'a>) -> Self::Item<'a> {
        EventWriter::new(execution_context.event_queue)
    }
}
impl<'q, E> Parameter for EventReader<'q, E>
where
    E: 'static,
{
    type Item<'a> = EventReader<'a, E>;

    fn fetch<'a>(execution_context: &'a ExecutionContext<'a>) -> Self::Item<'a> {
        EventReader::new(execution_context.event_queue.pending_events())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_set_add_system() {
        let mut system_set = SystemSet::new();
        system_set.add_system(|| {});
        system_set.add_system(|_command_buffer: &CommandBuffer| {});
        assert_eq!(system_set.iter_mut().count(), 2);
    }
}
