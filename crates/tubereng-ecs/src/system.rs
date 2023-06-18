use std::{marker::PhantomData, ops::DerefMut};

use crate::{
    commands::CommandBuffer,
    entity::EntityStore,
    query::{Query, Q},
};

pub struct ExecutionContext<'a> {
    pub(crate) command_buffer: &'a CommandBuffer,
    pub(crate) entity_store: &'a EntityStore,
}

pub trait System {
    fn run<'a>(&'a mut self, ctx: &'a ExecutionContext<'a>);
}

pub struct Function<F, M>
where
    F: SystemFn<M>,
{
    system_fn: F,
    _marker: PhantomData<M>,
}

impl<F, M> System for Function<F, M>
where
    F: SystemFn<M>,
{
    fn run<'a>(&'a mut self, ctx: &'a ExecutionContext<'a>) {
        let parameters = F::Parameter::fetch(ctx);
        self.system_fn.run(parameters);
    }
}

pub trait Into<M> {
    type SystemType: System;
    fn into(system: Self) -> Self::SystemType;
}

impl System for Box<dyn System> {
    fn run<'a>(&'a mut self, ctx: &'a ExecutionContext<'a>) {
        self.deref_mut().run(ctx);
    }
}

impl Into<()> for Box<dyn System> {
    type SystemType = Self;

    fn into(system: Self) -> Self::SystemType {
        system
    }
}

impl<F, M> Into<M> for F
where
    F: SystemFn<M>,
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
    F: FnMut() + 'static,
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
            for<'a> &'a mut F: FnMut($head, $($tail,)*) + FnMut($head::Item<'a>, $($tail::Item<'a>,)*),
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

impl<'q, QD> Parameter for Q<'q, QD>
where
    QD: for<'a> Query<'a>,
{
    type Item<'a> = Q<'a, QD>;

    fn fetch<'a>(execution_context: &'a ExecutionContext<'a>) -> Self::Item<'a> {
        Q::new(execution_context.entity_store)
    }
}
