use std::any::TypeId;
use std::cell::{Ref, RefMut};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use crate::commands::CommandQueue;
use crate::relationship::Relationship;
use crate::{query, ComponentStores, Ecs, EntityId, Storage};

pub struct Schedule {
    stages: Vec<TypeId>,
    stages_systems: HashMap<TypeId, Vec<System>>,
}

impl Schedule {
    #[must_use]
    pub fn new() -> Self {
        Self {
            stages: vec![],
            stages_systems: HashMap::new(),
        }
    }

    /// Run the systems registered in the schedule
    ///
    /// # Panics
    ///
    /// Will panic if the systems of a stage cannot be found
    pub fn run_systems(&mut self, ecs: &mut Ecs) {
        for stage in &self.stages {
            let systems = self.stages_systems.get_mut(stage).unwrap();
            for system in systems.iter_mut() {
                system.run(ecs);
            }
        }
    }

    pub fn add_system<Stage, F, S>(&mut self, _stage: &Stage, system: F)
    where
        Stage: 'static,
        S: 'static,
        F: 'static + Into<S>,
    {
        let stage = TypeId::of::<Stage>();
        if !self.stages_systems.contains_key(&stage) {
            self.stages.push(stage);
        }

        self.stages_systems
            .entry(TypeId::of::<Stage>())
            .or_default()
            .push(system.into_system());
    }

    /// Registers a system to the schedule for a given stage.
    /// If the stage doesn't exist, it is created and will run
    /// after the already registered stages.
    pub fn register_system_for_stage<S>(&mut self, system: System)
    where
        S: 'static,
    {
        let stage_id = TypeId::of::<S>();
        if let Entry::Vacant(entry) = self.stages_systems.entry(stage_id) {
            entry.insert(vec![]);
            self.stages.push(stage_id);
        }

        // SAFETY: If the entry was vacant we created it, so it must be here
        unsafe {
            self.stages_systems
                .get_mut(&TypeId::of::<S>())
                .unwrap_unchecked()
                .push(system);
        }
    }
}

impl Default for Schedule {
    fn default() -> Self {
        Self::new()
    }
}

type SystemFn = Box<dyn Fn(&mut CommandQueue, &Storage)>;

pub struct System {
    system_fn: SystemFn,
}

impl System {
    pub fn run(&self, ecs: &mut Ecs) {
        (self.system_fn)(&mut ecs.command_queue, &mut ecs.storage);
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
                    system_fn: Box::new(move |command_queue, storage| (self)($head::provide(command_queue, storage).unwrap(), $($tail::provide(command_queue, storage).unwrap(),)*)),
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
    fn provide<'a>(command_queue: &'a CommandQueue, storage: &'a Storage)
        -> Option<Self::Type<'a>>;
}

impl Argument for () {
    type Type<'a> = ();

    fn provide<'a>(
        _command_queue: &'a CommandQueue,
        _storage: &'a Storage,
    ) -> Option<Self::Type<'a>> {
        Some(())
    }
}

impl Argument for &Storage {
    type Type<'a> = &'a Storage;

    fn provide<'a>(
        _command_queue: &'a CommandQueue,
        storage: &'a Storage,
    ) -> Option<Self::Type<'a>> {
        Some(storage)
    }
}

impl<A> Argument for Option<A>
where
    A: Argument,
{
    type Type<'a> = Option<A::Type<'a>>;

    fn provide<'a>(
        command_queue: &'a CommandQueue,
        storage: &'a Storage,
    ) -> Option<Self::Type<'a>> {
        Some(A::provide(command_queue, storage))
    }
}

pub struct Q<'ecs, QD>
where
    QD: query::Definition,
{
    state: query::State<'ecs, QD>,
    _marker: PhantomData<QD>,
}

impl<'ecs, QD> Q<'ecs, QD>
where
    QD: query::Definition,
{
    #[must_use]
    pub fn new(
        component_stores: &'ecs ComponentStores,
        deleted_entities: &'ecs [EntityId],
        entity_count: usize,
    ) -> Self {
        let state = query::State::new(component_stores, deleted_entities, entity_count);
        Self {
            state,
            _marker: PhantomData,
        }
    }

    pub fn first(&mut self) -> Option<QD::Item<'_>> {
        self.state.iter().next()
    }

    pub fn first_with_id(&mut self) -> Option<(EntityId, QD::Item<'_>)> {
        self.state.iter_with_ids().next()
    }
    pub fn iter<'a>(&'a mut self) -> query::Iter<'ecs, 'a, QD> {
        self.state.iter()
    }
    pub fn iter_with_ids<'a>(&'a mut self) -> query::IterWithIds<'ecs, 'a, QD> {
        self.state.iter_with_ids()
    }
}

impl<'ecs, QD> Argument for Q<'ecs, QD>
where
    QD: query::Definition,
{
    type Type<'a> = Q<'a, QD>;

    fn provide<'a>(
        _command_queue: &'a CommandQueue,
        storage: &'a Storage,
    ) -> Option<Self::Type<'a>> {
        Some(Q::new(
            &storage.component_stores,
            &storage.deleted_entities,
            storage.entity_count(),
        ))
    }
}

impl Argument for &CommandQueue {
    type Type<'a> = &'a CommandQueue;
    fn provide<'a>(
        command_queue: &'a CommandQueue,
        _storage: &'a Storage,
    ) -> Option<Self::Type<'a>> {
        Some(command_queue)
    }
}

pub struct Rel<'a, R>(&'a Relationship, PhantomData<&'a R>);
impl<'a, R> Deref for Rel<'a, R> {
    type Target = &'a Relationship;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<R> Argument for Rel<'_, R>
where
    R: 'static,
{
    type Type<'a> = Rel<'a, R>;

    fn provide<'a>(
        _command_queue: &'a CommandQueue,
        storage: &'a Storage,
    ) -> Option<Self::Type<'a>> {
        Some(Rel(storage.relationship::<R>()?, PhantomData))
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

    fn provide<'a>(
        _command_queue: &'a CommandQueue,
        storage: &'a Storage,
    ) -> Option<Self::Type<'a>> {
        Some(Res(Ref::map(
            storage.resources.get(&TypeId::of::<T>()).as_ref()?.borrow(),
            |r| r.downcast_ref::<T>().unwrap(),
        )))
    }
}
pub struct ResMut<'a, T>(RefMut<'a, T>);
impl<'a, T> Deref for ResMut<'a, T> {
    type Target = RefMut<'a, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<'a, T> DerefMut for ResMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: 'static> Argument for ResMut<'_, T> {
    type Type<'a> = ResMut<'a, T>;

    fn provide<'a>(
        _command_queue: &'a CommandQueue,
        storage: &'a Storage,
    ) -> Option<Self::Type<'a>> {
        Some(ResMut(RefMut::map(
            storage
                .resources
                .get(&TypeId::of::<T>())
                .as_ref()?
                .borrow_mut(),
            |r| r.downcast_mut::<T>().unwrap(),
        )))
    }
}

#[cfg(test)]
mod tests {
    use crate::{relationship::ChildOf, Ecs};

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

    #[derive(Debug, PartialEq, Eq)]
    struct MyResource;
    #[test]
    fn ecs_optional_arg() {
        let mut ecs = Ecs::new();
        ecs.run_single_run_system(
            &(|res: Option<Res<MyResource>>| assert!(res.is_none())).into_system(),
        );

        ecs.insert_resource(MyResource);
        ecs.run_single_run_system(
            &(|res: Option<Res<MyResource>>| assert!(res.is_some())).into_system(),
        );
    }

    #[test]
    fn ecs_relationship() {
        let mut ecs = Ecs::new();
        let a = ecs.insert(());
        let b = ecs.insert(());
        ecs.insert_relationship::<ChildOf>(b, a);

        ecs.run_single_run_system(
            &(move |rel: Rel<ChildOf>| {
                assert!(rel.sources(a).unwrap().contains(&b));
            })
            .into_system(),
        );
    }
}
