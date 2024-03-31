use std::any::TypeId;
use std::cell::{Ref, RefMut};
use std::ops::Deref;

use crate::commands::CommandQueue;
use crate::{ComponentStores, Resources};

type SystemFn = Box<dyn Fn(&mut CommandQueue, &Resources)>;

pub struct System {
    system_fn: SystemFn,
}

impl System {
    pub fn run(
        &self,
        _component_stores: &mut ComponentStores,
        resources: &mut Resources,
        command_queue: &mut CommandQueue,
    ) {
        (self.system_fn)(command_queue, resources);
    }
}

pub struct Noop;
impl<A> Into<A> for Noop {
    fn into_system(self) -> System {
        System {
            system_fn: Box::new(|_, _| {}),
        }
    }
}

pub trait Into<A>: BoxedInto<A> {
    fn into_system(self) -> System;
}

pub trait BoxedInto<A> {
    fn into_system(self: Box<Self>) -> System;
}

impl<A, T> BoxedInto<A> for T
where
    T: Into<A>,
{
    fn into_system(self: Box<Self>) -> System {
        <Self as Into<A>>::into_system(*self)
    }
}

impl<A> Into<A> for System {
    fn into_system(self) -> System {
        self
    }
}

impl<A> Into<A> for Box<dyn Into<A>> {
    fn into_system(self) -> System {
        self.into_system()
    }
}

impl<F> Into<()> for F
where
    F: 'static + Fn(),
{
    fn into_system(self) -> System {
        System {
            system_fn: Box::new(move |_, _| (self)()),
        }
    }
}

macro_rules! impl_into_for_tuples {
    ($head:tt, $($tail:tt,)*) => {
        impl<FN, $head, $($tail,)*> Into<($head, $($tail,)*)> for FN
        where
            for<'a> FN: 'static + Fn($head, $($tail,)*) + Fn($head::Type<'a>, $($tail::Type<'a>,)*),
            $head: Argument,
            $($tail: Argument,)*
        {
            fn into_system(self) -> System {
                System {
                    system_fn: Box::new(move |command_queue, resources| (self)($head::provide(command_queue, resources), $($tail::provide(command_queue, resources),)*)),
                }
            }
        }

        impl_into_for_tuples!($($tail,)*);
    };
    () => {}
}

impl_into_for_tuples!(F, E, D, C, B, A,);

pub trait Argument {
    type Type<'a>;
    fn provide<'a>(command_queue: &'a CommandQueue, resources: &'a Resources) -> Self::Type<'a>;
}

impl Argument for () {
    type Type<'a> = ();

    fn provide<'a>(_command_queue: &'a CommandQueue, _resources: &'a Resources) -> Self::Type<'a> {}
}

impl Argument for &CommandQueue {
    type Type<'a> = &'a CommandQueue;
    fn provide<'a>(command_queue: &'a CommandQueue, _resources: &'a Resources) -> Self::Type<'a> {
        command_queue
    }
}

pub struct Res<'a, T>(Ref<'a, T>);
impl<'a, T> Deref for Res<'a, T> {
    type Target = Ref<'a, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: 'static> Argument for Res<'_, T> {
    type Type<'a> = Res<'a, T>;

    fn provide<'a>(_command_queue: &'a CommandQueue, resources: &'a Resources) -> Self::Type<'a> {
        Res(Ref::map(
            resources.get(&TypeId::of::<T>()).as_ref().unwrap().borrow(),
            |r| r.downcast_ref::<T>().unwrap(),
        ))
    }
}
pub struct ResMut<'a, T>(RefMut<'a, T>);
impl<'a, T> Deref for ResMut<'a, T> {
    type Target = RefMut<'a, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: 'static> Argument for ResMut<'_, T> {
    type Type<'a> = ResMut<'a, T>;

    fn provide<'a>(_command_queue: &'a CommandQueue, resources: &'a Resources) -> Self::Type<'a> {
        ResMut(RefMut::map(
            resources
                .get(&TypeId::of::<T>())
                .as_ref()
                .unwrap()
                .borrow_mut(),
            |r| r.downcast_mut::<T>().unwrap(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::Ecs;

    use super::*;

    #[derive(Debug)]
    struct Player;
    #[derive(Debug)]
    struct Enemy;
    #[derive(Debug, PartialEq)]
    struct Health(i32);
    #[derive(Debug, PartialEq)]
    struct Position {
        x: i32,
        y: i32,
    }

    #[test]
    fn ecs_run_single_system() {
        let mut ecs = Ecs::new();
        ecs.run_single_run_system(
            &(|command_queue: &CommandQueue| {
                command_queue.insert((Player, Health(10), Position { x: 3, y: 5 }));
                command_queue.insert((Enemy, Health(5), Position { x: 5, y: 9 }));
                command_queue.insert((Enemy, Health(2), Position { x: 7, y: 12 }));
            })
            .into_system(),
        );
        assert_eq!(ecs.entity_count(), 3);
    }
}
