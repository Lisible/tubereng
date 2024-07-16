#![warn(clippy::pedantic)]

use log::trace;
use relationship::{Relationship, Relationships};
use std::{
    alloc::Layout,
    any::{Any, TypeId},
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
};

use commands::CommandQueue;
use component_store::{drop_fn_of, ComponentRef, ComponentRefMut, ComponentStore};

mod bitset;
pub mod commands;
mod component_store;
pub mod query;
pub mod relationship;
pub mod system;

pub type EntityId = usize;
pub type ComponentStores = HashMap<TypeId, ComponentStore>;
pub type Resources = HashMap<TypeId, RefCell<Box<dyn Any>>>;

const MAX_ENTITY_COUNT: usize = 1024;
pub struct Storage {
    next_entity_id: EntityId,
    deleted_entities: Vec<EntityId>,
    component_stores: ComponentStores,
    relationships: Relationships,
    resources: Resources,
}

impl Default for Storage {
    fn default() -> Self {
        Self::new()
    }
}

impl Storage {
    #[must_use]
    pub fn new() -> Self {
        Self {
            next_entity_id: 0,
            deleted_entities: vec![],
            component_stores: ComponentStores::new(),
            resources: Resources::new(),
            relationships: Relationships::new(),
        }
    }

    #[must_use]
    pub fn next_entity_id(&self) -> usize {
        self.next_entity_id
    }

    #[must_use]
    pub fn entity_count(&self) -> usize {
        self.next_entity_id - self.deleted_entities.len()
    }

    pub fn clear_dirty_flags(&mut self) {
        for component_store in self.component_stores.values_mut() {
            component_store.clear_dirty_bitset();
        }
    }

    #[must_use]
    pub fn dirty_state<C: 'static>(&self, entity_id: EntityId) -> bool {
        let Some(component_store) = self.component_stores.get(&TypeId::of::<C>()) else {
            return false;
        };

        component_store.dirty(entity_id)
    }

    pub fn insert<ED>(&mut self, entity_definition: ED) -> EntityId
    where
        ED: EntityDefinition,
    {
        let entity_id = self.allocate_entity();
        trace!("Inserting entity {entity_id} with definition {entity_definition:?}");
        entity_definition.write_into_component_stores(entity_id, &mut self.component_stores);
        entity_id
    }

    pub fn insert_component<C: 'static>(&mut self, entity_id: EntityId, component: C) {
        let component_store = self
            .component_stores
            .entry(TypeId::of::<C>())
            .or_insert(ComponentStore::new(Layout::new::<C>(), drop_fn_of::<C>));
        component_store.store(entity_id, component);
    }

    pub fn remove_component<C: 'static>(&mut self, entity_id: EntityId) {
        let Some(component_store) = self.component_stores.get_mut(&TypeId::of::<C>()) else {
            return;
        };

        component_store.delete(entity_id);
    }

    pub fn delete(&mut self, entity_id: EntityId) {
        for component_store in self.component_stores.values_mut() {
            component_store.delete(entity_id);
        }
        self.deleted_entities.push(entity_id);
    }

    pub fn insert_resource<R>(&mut self, resource: R)
    where
        R: Any,
    {
        self.resources
            .insert(TypeId::of::<R>(), RefCell::new(Box::new(resource)));
    }

    /// Fetches a resource from the Ecs
    ///
    /// # Panics
    ///
    /// Will panic if the resource can't be downcasted to its actual type
    #[must_use]
    pub fn resource<R: Any>(&self) -> Option<Ref<'_, R>> {
        Some(Ref::map(
            self.resources.get(&TypeId::of::<R>())?.borrow(),
            |r| r.downcast_ref::<R>().expect("Couldn't downcast resource"),
        ))
    }

    /// Fetches a mutable resource from the Ecs
    ///
    /// # Panics
    ///
    /// Will panic if the resource can't be downcasted to its actual type
    #[must_use]
    pub fn resource_mut<R: Any>(&self) -> Option<RefMut<'_, R>> {
        Some(RefMut::map(
            self.resources.get(&TypeId::of::<R>())?.borrow_mut(),
            |r| r.downcast_mut::<R>().expect("Couldn't downcast resource"),
        ))
    }

    pub fn insert_relationship<R: 'static>(&mut self, source: EntityId, target: EntityId) {
        self.relationships.insert::<R>(source, target);
    }

    pub fn define_relationship<R: 'static>(&mut self) {
        self.relationships.define::<R>();
    }

    #[must_use]
    pub fn relationship<R: 'static>(&self) -> Option<&Relationship> {
        self.relationships.get::<R>()
    }

    #[must_use]
    pub fn component<C>(&self, entity_id: EntityId) -> Option<ComponentRef<C>>
    where
        C: 'static,
    {
        if self.deleted_entities.contains(&entity_id) {
            return None;
        }

        self.component_stores
            .get(&TypeId::of::<C>())?
            .get(entity_id)
    }

    #[must_use]
    pub fn component_mut<C>(&self, entity_id: EntityId) -> Option<ComponentRefMut<C>>
    where
        C: 'static,
    {
        if self.deleted_entities.contains(&entity_id) {
            return None;
        }

        self.component_stores
            .get(&TypeId::of::<C>())?
            .get_mut(entity_id)
    }

    #[must_use]
    pub fn query<QD>(&self) -> query::State<QD>
    where
        QD: query::Definition,
    {
        query::State::new(
            &self.component_stores,
            &self.deleted_entities,
            self.next_entity_id - 1,
        )
    }

    fn allocate_entity(&mut self) -> EntityId {
        if let Some(entity_id) = self.deleted_entities.pop() {
            return entity_id;
        }

        let entity_id = self.next_entity_id;
        self.next_entity_id += 1;
        entity_id
    }
}

pub struct Ecs {
    storage: Storage,
    command_queue: CommandQueue,
}

impl Ecs {
    #[must_use]
    pub fn new() -> Self {
        Ecs {
            storage: Storage::new(),
            command_queue: CommandQueue::new(0, &[]),
        }
    }

    #[must_use]
    pub fn entity_count(&self) -> usize {
        self.storage.entity_count()
    }

    /// Inserts a new entity with its components into the Ecs
    pub fn insert<ED>(&mut self, entity_definition: ED) -> EntityId
    where
        ED: EntityDefinition,
    {
        self.storage.insert(entity_definition)
    }

    pub fn insert_component<C: 'static>(&mut self, entity_id: EntityId, component: C) {
        self.storage.insert_component(entity_id, component);
    }

    pub fn remove_component<C: 'static>(&mut self, entity_id: EntityId) {
        self.storage.remove_component::<C>(entity_id);
    }

    /// Deletes the entity with the given id
    pub fn delete(&mut self, entity_id: EntityId) {
        self.storage.delete(entity_id);
    }

    /// Inserts a resource into the Ecs, replaces it if already present
    pub fn insert_resource<R>(&mut self, resource: R)
    where
        R: Any,
    {
        self.storage.insert_resource(resource);
    }

    pub fn insert_relationship<R: 'static>(&mut self, source: EntityId, target: EntityId) {
        self.storage.insert_relationship::<R>(source, target);
    }

    pub fn define_relationship<R: 'static>(&mut self) {
        self.storage.define_relationship::<R>();
    }

    pub fn relationship<R: 'static>(&self) -> Option<&Relationship> {
        self.storage.relationship::<R>()
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
        self.storage.resource()
    }

    /// Retrieves a ``RefMut`` to a stored resource or None if its not found
    ///
    /// # Panics
    ///
    /// Will panic if the downcasting to the resource type
    pub fn resource_mut<R: Any>(&self) -> Option<RefMut<'_, R>> {
        self.storage.resource_mut()
    }

    /// Returns an immutable reference to a component in the Ecs, or `None` if not found.
    #[must_use]
    pub fn component<C>(&self, entity_id: EntityId) -> Option<ComponentRef<C>>
    where
        C: 'static,
    {
        self.storage.component(entity_id)
    }

    /// Returns a mutable reference to a component in the Ecs, or `None` if not found.
    #[must_use]
    pub fn component_mut<C>(&self, entity_id: EntityId) -> Option<ComponentRefMut<C>>
    where
        C: 'static,
    {
        self.storage.component_mut(entity_id)
    }

    pub fn query<QD>(&mut self) -> query::State<QD>
    where
        QD: query::Definition,
    {
        self.storage.query()
    }

    pub fn run_single_run_system(&mut self, system: &system::System) {
        system.run(self);
        self.process_command_queue();
    }

    pub fn clear_dirty_flags(&mut self) {
        self.storage.clear_dirty_flags();
    }

    pub fn process_command_queue(&mut self) {
        let mut command_queue =
            CommandQueue::new(self.storage.next_entity_id, &self.storage.deleted_entities);
        std::mem::swap(&mut self.command_queue, &mut command_queue);
        for mut command in command_queue {
            command.apply(self);
        }
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
                    .or_insert_with(|| ComponentStore::new(Layout::new::<$head>(), drop_fn_of::<$head>))
                    .store(entity_id, self.$head_i);
                $(component_stores
                    .entry(TypeId::of::<$tail>())
                    .or_insert_with(|| ComponentStore::new(Layout::new::<$tail>(), drop_fn_of::<$tail>))
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

        assert_eq!(
            ecs.component::<Health>(player).as_deref(),
            Some(&Health(10))
        );
        assert_eq!(
            ecs.component::<Health>(first_enemy).as_deref(),
            Some(&Health(5))
        );
        assert_eq!(
            ecs.component::<Position>(second_enemy).as_deref(),
            Some(&Position { x: 7, y: 12 })
        );
    }

    #[test]
    fn ecs_component_mut() {
        let mut ecs = Ecs::new();
        let enemy = ecs.insert((Enemy, Health(5), Position { x: 5, y: 9 }));
        {
            let mut enemy_health = ecs.component_mut::<Health>(enemy).unwrap();
            enemy_health.0 -= 1;
        }
        assert_eq!(ecs.component::<Health>(enemy).as_deref(), Some(&Health(4)));
    }

    #[test]
    fn ecs_query() {
        let mut ecs = Ecs::new();
        let _ = ecs.insert((Player, Health(10), Position { x: 3, y: 5 }));
        let _ = ecs.insert((Enemy, Health(5), Position { x: 5, y: 9 }));
        let _ = ecs.insert((Enemy, Health(2), Position { x: 7, y: 12 }));

        let mut health_query = ecs.query::<&Health>();
        let mut health_query_iter = health_query.iter();
        assert_eq!(health_query_iter.next().as_deref(), Some(&Health(10)));
        assert_eq!(health_query_iter.next().as_deref(), Some(&Health(5)));
        assert_eq!(health_query_iter.next().as_deref(), Some(&Health(2)));
    }

    #[test]
    fn ecs_query_multiple() {
        let mut ecs = Ecs::new();
        let _ = ecs.insert((Player, Health(10), Position { x: 3, y: 5 }));
        let _ = ecs.insert((Enemy, Health(5), Position { x: 5, y: 9 }));
        let _ = ecs.insert((Enemy, Health(2), Position { x: 7, y: 12 }));
        let mut health_pos_query = ecs.query::<(&Health, &Position)>();
        let mut health_pos_query_iter = health_pos_query.iter();

        let first_health_pos = health_pos_query_iter.next().unwrap();
        assert_eq!(*first_health_pos.0, Health(10));
        assert_eq!(*first_health_pos.1, Position { x: 3, y: 5 });

        let second_health_pos = health_pos_query_iter.next().unwrap();
        assert_eq!(*second_health_pos.0, Health(5));
        assert_eq!(*second_health_pos.1, Position { x: 5, y: 9 });

        let third_health_pos = health_pos_query_iter.next().unwrap();
        assert_eq!(*third_health_pos.0, Health(2));
        assert_eq!(*third_health_pos.1, Position { x: 7, y: 12 });
    }

    #[test]
    fn ecs_query_mutable() {
        let mut ecs = Ecs::new();
        let player = ecs.insert((Player, Health(10), Position { x: 3, y: 5 }));
        let first_enemy = ecs.insert((Enemy, Health(5), Position { x: 5, y: 9 }));
        let second_enemy = ecs.insert((Enemy, Health(2), Position { x: 7, y: 12 }));

        for (_, mut enemy_health) in ecs.query::<(&Enemy, &mut Health)>().iter() {
            let Health(enemy_health) = &mut *enemy_health;
            *enemy_health -= 1;
        }

        assert_eq!(
            ecs.component::<Health>(player).as_deref(),
            Some(&Health(10))
        );
        assert_eq!(
            ecs.component::<Health>(first_enemy).as_deref(),
            Some(&Health(4))
        );
        assert_eq!(
            ecs.component::<Health>(second_enemy).as_deref(),
            Some(&Health(1))
        );
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

    #[test]
    fn ecs_insert_relationship() {
        struct ChildOf;
        let mut ecs = Ecs::new();
        let entity_a = ecs.insert(());
        let entity_b = ecs.insert(());
        ecs.insert_relationship::<ChildOf>(entity_b, entity_a);
        assert!(ecs
            .relationship::<ChildOf>()
            .unwrap()
            .sources(entity_a)
            .unwrap()
            .contains(&entity_b));
    }

    #[test]
    fn storage_clear_dirty_flags() {
        let mut storage = Storage::new();
        storage.insert((Health(23),));
        storage.clear_dirty_flags();

        let mut health = storage.component_mut::<Health>(0).unwrap();
        assert!(!storage.component_stores[&TypeId::of::<Health>()].dirty(0));
        health.0 = 22;
        assert!(storage.component_stores[&TypeId::of::<Health>()].dirty(0));
        storage.clear_dirty_flags();
        assert!(!storage.component_stores[&TypeId::of::<Health>()].dirty(0));
    }
}
