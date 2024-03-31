use crate::commands::CommandQueue;
use crate::ComponentStores;

pub struct System {
    system_fn: Box<dyn Fn(&mut CommandQueue)>,
}

impl System {
    pub fn run(&self, _component_stores: &mut ComponentStores, command_queue: &mut CommandQueue) {
        (self.system_fn)(command_queue);
    }
}

pub struct Noop;
impl<A> Into<A> for Noop {
    fn into_system(self) -> System {
        System {
            system_fn: Box::new(|_| {}),
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
            system_fn: Box::new(move |_| (self)()),
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
                    system_fn: Box::new(move |command_queue| (self)($head::provide(command_queue), $($tail::provide(command_queue),)*)),
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
    fn provide(command_queue: &CommandQueue) -> Self::Type<'_>;
}

impl Argument for () {
    type Type<'a> = ();

    fn provide(_command_queue: &CommandQueue) -> Self::Type<'_> {}
}

impl Argument for &CommandQueue {
    type Type<'a> = &'a CommandQueue;
    fn provide(command_queue: &CommandQueue) -> Self::Type<'_> {
        command_queue
    }
}

#[cfg(test)]
mod tests {
    use crate::{commands::InsertEntity, Ecs};

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
                command_queue.push_command(InsertEntity::new((
                    Player,
                    Health(10),
                    Position { x: 3, y: 5 },
                )));
                command_queue.push_command(InsertEntity::new((
                    Enemy,
                    Health(5),
                    Position { x: 5, y: 9 },
                )));
                command_queue.push_command(InsertEntity::new((
                    Enemy,
                    Health(2),
                    Position { x: 7, y: 12 },
                )));
            })
            .into_system(),
        );
        assert_eq!(ecs.entity_count(), 3);
    }
}
