#![warn(clippy::pedantic)]

use log::trace;
use std::{
    alloc::Layout,
    any::{Any, TypeId},
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
};

use commands::CommandQueue;
use component_store::ComponentStore;

mod bitset;
pub mod commands;
mod component_store;
pub mod query;
pub mod system;

pub type EntityId = usize;
pub type ComponentStores = HashMap<TypeId, ComponentStore>;
pub type Resources = HashMap<TypeId, RefCell<Box<dyn Any>>>;

const MAX_ENTITY_COUNT: usize = 1024;
pub struct Ecs {
    next_entity_id: EntityId,
    component_stores: ComponentStores,
    resources: Resources,
    command_queue: CommandQueue,
    systems: Vec<system::System>,
}

impl Ecs {
    #[must_use]
    pub fn new() -> Self {
        Ecs {
            next_entity_id: 0,
            component_stores: ComponentStores::new(),
            resources: Resources::new(),
            command_queue: CommandQueue::new(),
            systems: vec![],
        }
    }

    #[must_use]
    pub fn entity_count(&self) -> usize {
        self.next_entity_id
    }

    /// Inserts a new entity with its components into the Ecs
    pub fn insert<ED>(&mut self, entity_definition: ED) -> EntityId
    where
        ED: EntityDefinition,
    {
        let entity_id = self.allocate_entity();
        trace!("Inserting entity {entity_id} with definition {entity_definition:?}");
        entity_definition.write_into_component_stores(entity_id, &mut self.component_stores);
        entity_id
    }

    /// Inserts a resource into the Ecs, replaces it if already present
    pub fn insert_resource<R>(&mut self, resource: R)
    where
        R: Any,
    {
        self.resources
            .insert(TypeId::of::<R>(), RefCell::new(Box::new(resource)));
    }

    pub fn command_queue(&self) -> &CommandQueue {
        &self.command_queue
    }

    /// Retrieves a ``Ref`` to a stored resource or None if its not found
    ///
    /// # Panics
    ///
    /// Will panic if the downcasting to the resource type
    pub fn resource<R: Any>(&self) -> Option<Ref<'_, R>> {
        Some(Ref::map(
            self.resources.get(&TypeId::of::<R>())?.borrow(),
            |r| r.downcast_ref::<R>().expect("Couldn't downcast resource"),
        ))
    }

    /// Retrieves a ``RefMut`` to a stored resource or None if its not found
    ///
    /// # Panics
    ///
    /// Will panic if the downcasting to the resource type
    pub fn resource_mut<R: Any>(&self) -> Option<RefMut<'_, R>> {
        Some(RefMut::map(
            self.resources.get(&TypeId::of::<R>())?.borrow_mut(),
            |r| r.downcast_mut::<R>().expect("Couldn't downcast resource"),
        ))
    }

    /// Returns an immutable reference to a component in the Ecs, or `None` if not found.
    #[must_use]
    pub fn component<C>(&self, entity_id: EntityId) -> Option<&C>
    where
        C: 'static,
    {
        self.component_stores
            .get(&TypeId::of::<C>())?
            .get(entity_id)
    }

    /// Returns a mutable reference to a component in the Ecs, or `None` if not found.
    #[must_use]
    pub fn component_mut<C>(&self, entity_id: EntityId) -> Option<&mut C>
    where
        C: 'static,
    {
        self.component_stores
            .get(&TypeId::of::<C>())?
            .get_mut(entity_id)
    }

    pub fn query<QD>(&mut self) -> query::State<QD>
    where
        QD: query::Definition,
    {
        query::State::new()
    }

    pub fn run_single_run_system(&mut self, system: &system::System) {
        system.run(
            &mut self.component_stores,
            &mut self.resources,
            &mut self.command_queue,
        );
        self.process_command_queue();
    }

    pub fn run_systems(&mut self) {
        for system in &self.systems {
            system.run(
                &mut self.component_stores,
                &mut self.resources,
                &mut self.command_queue,
            );
        }
    }

    pub fn register_system<F, A>(&mut self, system: F)
    where
        F: system::Into<A>,
    {
        self.insert_system(system.into_system());
    }

    fn insert_system(&mut self, system: system::System) {
        trace!("Registering system @{:?}", std::ptr::addr_of!(system));
        self.systems.push(system);
    }

    fn process_command_queue(&mut self) {
        let mut command_queue = CommandQueue::new();
        std::mem::swap(&mut self.command_queue, &mut command_queue);
        for mut command in command_queue {
            command.apply(self);
        }
    }

    fn allocate_entity(&mut self) -> EntityId {
        let entity_id = self.next_entity_id;
        self.next_entity_id += 1;
        entity_id
    }
}

impl Default for Ecs {
    fn default() -> Self {
        Self::new()
    }
}

pub trait EntityDefinition: BoxedEntityDefinition + std::fmt::Debug {
    fn write_into_component_stores(
        self,
        entity_id: EntityId,
        component_stores: &mut ComponentStores,
    );
}

pub trait BoxedEntityDefinition {
    fn write_into_component_stores(
        self: Box<Self>,
        entity_id: EntityId,
        component_stores: &mut ComponentStores,
    );
}

impl<T> BoxedEntityDefinition for T
where
    T: EntityDefinition,
{
    fn write_into_component_stores(
        self: Box<T>,
        entity_id: EntityId,
        component_stores: &mut ComponentStores,
    ) {
        <Self as EntityDefinition>::write_into_component_stores(*self, entity_id, component_stores);
    }
}

impl EntityDefinition for Box<dyn EntityDefinition> {
    fn write_into_component_stores(
        self,
        entity_id: EntityId,
        component_stores: &mut ComponentStores,
    ) {
        <dyn EntityDefinition as BoxedEntityDefinition>::write_into_component_stores(
            self,
            entity_id,
            component_stores,
        );
    }
}

impl EntityDefinition for () {
    fn write_into_component_stores(
        self,
        _entity_id: EntityId,
        _component_stores: &mut ComponentStores,
    ) {
    }
}

macro_rules! impl_entity_definition_for_tuple {
    ($head:ident: $head_i:tt, $($tail:ident: $tail_i:tt,)*) => {
        impl<$head, $($tail,)*> EntityDefinition for ($head, $($tail,)*)
        where
            $head: 'static + std::fmt::Debug,
            $($tail: 'static + std::fmt::Debug,)*
        {
            fn write_into_component_stores(
                self,
                entity_id: EntityId,
                component_stores: &mut ComponentStores,
            ) {
                component_stores
                    .entry(TypeId::of::<$head>())
                    .or_insert_with(|| ComponentStore::new(Layout::new::<$head>()))
                    .store(entity_id, self.$head_i);
                $(component_stores
                    .entry(TypeId::of::<$tail>())
                    .or_insert_with(|| ComponentStore::new(Layout::new::<$tail>()))
                    .store(entity_id, self.$tail_i);)*
            }
        }
    };
    () => {}
}

// TODO; replace these with a proc macro
impl_entity_definition_for_tuple!(A: 0,);
impl_entity_definition_for_tuple!(A: 0, B: 1,);
impl_entity_definition_for_tuple!(A: 0, B: 1, C: 2,);
impl_entity_definition_for_tuple!(A: 0, B: 1, C: 2, D: 3,);
impl_entity_definition_for_tuple!(A: 0, B: 1, C: 2, D: 3, E: 4,);
impl_entity_definition_for_tuple!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5,);

#[cfg(test)]
mod tests {

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
    fn ecs_new() {
        let ecs = Ecs::new();
        assert_eq!(ecs.entity_count(), 0);
    }

    #[test]
    fn ecs_insert() {
        let mut ecs = Ecs::new();
        ecs.insert((Player, Health(10), Position { x: 3, y: 5 }));
        assert_eq!(ecs.entity_count(), 1);
        ecs.insert((Enemy, Health(5), Position { x: 5, y: 9 }));
        ecs.insert((Enemy, Health(2), Position { x: 7, y: 12 }));
        assert_eq!(ecs.entity_count(), 3);
    }

    #[test]
    fn ecs_component() {
        let mut ecs = Ecs::new();
        let player = ecs.insert((Player, Health(10), Position { x: 3, y: 5 }));
        let first_enemy = ecs.insert((Enemy, Health(5), Position { x: 5, y: 9 }));
        let second_enemy = ecs.insert((Enemy, Health(2), Position { x: 7, y: 12 }));

        assert_eq!(ecs.component::<Health>(player), Some(&Health(10)));
        assert_eq!(ecs.component::<Health>(first_enemy), Some(&Health(5)));
        assert_eq!(
            ecs.component::<Position>(second_enemy),
            Some(&Position { x: 7, y: 12 })
        );
    }

    #[test]
    fn ecs_component_mut() {
        let mut ecs = Ecs::new();
        let enemy = ecs.insert((Enemy, Health(5), Position { x: 5, y: 9 }));
        let enemy_health = ecs.component_mut::<Health>(enemy).unwrap();
        enemy_health.0 -= 1;
        assert_eq!(ecs.component::<Health>(enemy), Some(&Health(4)));
    }

    #[test]
    fn ecs_query() {
        let mut ecs = Ecs::new();
        let _ = ecs.insert((Player, Health(10), Position { x: 3, y: 5 }));
        let _ = ecs.insert((Enemy, Health(5), Position { x: 5, y: 9 }));
        let _ = ecs.insert((Enemy, Health(2), Position { x: 7, y: 12 }));

        let mut health_query = ecs.query::<&Health>();
        let mut health_query_iter = health_query.iter(&ecs);
        assert_eq!(health_query_iter.next(), Some(&Health(10)));
        assert_eq!(health_query_iter.next(), Some(&Health(5)));
        assert_eq!(health_query_iter.next(), Some(&Health(2)));
    }

    #[test]
    fn ecs_query_multiple() {
        let mut ecs = Ecs::new();
        let _ = ecs.insert((Player, Health(10), Position { x: 3, y: 5 }));
        let _ = ecs.insert((Enemy, Health(5), Position { x: 5, y: 9 }));
        let _ = ecs.insert((Enemy, Health(2), Position { x: 7, y: 12 }));
        let mut health_pos_query = ecs.query::<(&Health, &Position)>();
        let mut health_pos_query_iter = health_pos_query.iter(&ecs);
        assert_eq!(
            health_pos_query_iter.next(),
            Some((&Health(10), &Position { x: 3, y: 5 }))
        );
        assert_eq!(
            health_pos_query_iter.next(),
            Some((&Health(5), &Position { x: 5, y: 9 }))
        );
        assert_eq!(
            health_pos_query_iter.next(),
            Some((&Health(2), &Position { x: 7, y: 12 }))
        );
    }

    #[test]
    fn ecs_query_mutable() {
        let mut ecs = Ecs::new();
        let player = ecs.insert((Player, Health(10), Position { x: 3, y: 5 }));
        let first_enemy = ecs.insert((Enemy, Health(5), Position { x: 5, y: 9 }));
        let second_enemy = ecs.insert((Enemy, Health(2), Position { x: 7, y: 12 }));

        for (_, enemy_health) in ecs.query::<(&Enemy, &mut Health)>().iter(&ecs) {
            let Health(enemy_health) = enemy_health;
            *enemy_health -= 1;
        }

        assert_eq!(ecs.component::<Health>(player), Some(&Health(10)));
        assert_eq!(ecs.component::<Health>(first_enemy), Some(&Health(4)));
        assert_eq!(ecs.component::<Health>(second_enemy), Some(&Health(1)));
    }

    #[test]
    fn ecs_resource() {
        #[derive(Debug, PartialEq)]
        struct SomeResource(i32);
        let mut ecs = Ecs::new();
        ecs.insert_resource(SomeResource(23));

        let r = ecs.resource::<SomeResource>().unwrap();
        assert_eq!(&*r, &SomeResource(23));
    }

    #[test]
    fn ecs_resource_mut() {
        #[derive(Debug, PartialEq)]
        struct SomeResource(i32);
        let mut ecs = Ecs::new();
        ecs.insert_resource(SomeResource(23));

        let mut r = ecs.resource_mut::<SomeResource>().unwrap();
        r.0 = 10;
        std::mem::drop(r);

        let r = ecs.resource::<SomeResource>().unwrap();
        assert_eq!(&*r, &SomeResource(10));
    }
}
