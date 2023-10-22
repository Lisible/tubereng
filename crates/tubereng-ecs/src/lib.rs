#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use entity::{EntityId, EntityStore};
use event::EventQueue;
use log::{info, trace};
use relationship::{Relationship, RelationshipStore};
use resource::Resources;
use std::{
    any::Any,
    cell::{Ref, RefMut},
    fmt::Debug,
};
use system::{ExecutionContext, System, SystemSet};

use commands::CommandBuffer;

pub mod commands;
pub mod entity;
pub mod event;
pub mod query;
pub mod relationship;
pub mod resource;
pub mod system;

pub struct Ecs {
    entity_store: EntityStore,
    relationship_store: RelationshipStore,
    pending_commands: CommandBuffer,
    setup_system: Option<Box<dyn System>>,
    system_sets: Vec<SystemSet>,
    event_queue: EventQueue,
    resources: Resources,
}
impl Ecs {
    #[must_use]
    pub fn new() -> Ecs {
        Self {
            entity_store: EntityStore::new(),
            pending_commands: CommandBuffer::new(),
            setup_system: None,
            system_sets: vec![],
            resources: Resources::new(),
            event_queue: EventQueue::new(),
            relationship_store: RelationshipStore::new(),
        }
    }

    #[must_use]
    pub fn entity_store(&self) -> &EntityStore {
        &self.entity_store
    }

    pub fn register_setup_system(&mut self, setup_system: Box<dyn System>) {
        info!("Registering setup system...");
        self.setup_system = Some(setup_system);
    }

    pub fn register_system_set(&mut self, system_set: SystemSet) {
        trace!("Registering a system set...");
        self.system_sets.push(system_set);
    }

    pub fn run_setup_system(&mut self) {
        info!("Running setup system...");
        if let Some(mut setup_system) = self.setup_system.take() {
            let ctx = ExecutionContext {
                command_buffer: &self.pending_commands,
                entity_store: &self.entity_store,
                relationship_store: &self.relationship_store,
                resources: &self.resources,
                event_queue: &self.event_queue,
            };
            setup_system.execute(&ctx);
        }
    }

    pub fn run_systems(&mut self) {
        self.event_queue.swap_and_clear();
        for system_set in &mut self.system_sets {
            for system in system_set.iter_mut() {
                let ctx = ExecutionContext {
                    command_buffer: &self.pending_commands,
                    entity_store: &self.entity_store,
                    relationship_store: &self.relationship_store,
                    resources: &self.resources,
                    event_queue: &self.event_queue,
                };
                system.execute(&ctx);
            }
        }
    }

    #[must_use]
    pub fn entity_count(&self) -> usize {
        self.entity_store.entity_count()
    }

    pub fn insert<ED>(&mut self, entity: ED) -> EntityId
    where
        ED: EntityDefinition,
    {
        self.entity_store.insert(entity)
    }

    pub fn insert_relationship<R>(&mut self, source: EntityId, target: EntityId)
    where
        R: Relationship,
    {
        self.relationship_store.insert::<R>(source, target);
    }

    pub fn insert_resource<R>(&mut self, resource: R)
    where
        R: Any,
    {
        self.resources.insert(resource);
    }

    #[must_use]
    pub fn resource<R>(&self) -> Option<Ref<R>>
    where
        R: Any,
    {
        self.resources.resource::<R>()
    }

    #[must_use]
    pub fn resource_mut<R>(&self) -> Option<RefMut<R>>
    where
        R: Any,
    {
        self.resources.resource_mut::<R>()
    }

    pub fn execute_pending_commands(&mut self) {
        let mut pending_commands = CommandBuffer::new();
        std::mem::swap(&mut self.pending_commands, &mut pending_commands);
        for mut command in pending_commands.flush_commands() {
            command.apply(self);
        }
    }

    #[must_use]
    pub fn event_queue_mut<'a>(
        &'a mut self,
    ) -> RefMut<'a, Vec<Box<(dyn std::any::Any + 'static)>>> {
        self.event_queue.pending_events_mut()
    }
}

impl Default for Ecs {
    fn default() -> Self {
        Self::new()
    }
}

pub trait EntityDefinition: Debug {
    fn write_into_entity_store(self, entity_store: &mut EntityStore, entity_id: EntityId);
}

macro_rules! impl_entity_definition_for_tuples {
    ($head:ident: $head_i:tt, $($tail:ident: $tail_i:tt,)*) => {
        impl<$head: 'static + Debug, $($tail: 'static + Debug,)*> EntityDefinition for ($head, $($tail,)*) {
            fn write_into_entity_store(self, entity_store: &mut EntityStore, entity_id: EntityId) {
                entity_store.write_component(entity_id, self.$head_i);
                $(entity_store.write_component(entity_id, self.$tail_i);)*
            }
        }

        impl_entity_definition_for_tuples!($($tail: $tail_i,)*);
    };

    () => {}
}

impl_entity_definition_for_tuples!(F: 5, E: 4, D: 3, C: 2, B: 1, A: 0,);

#[cfg(test)]
mod tests {

    use crate::{
        event::{EventReader, EventWriter},
        query::Q,
        system::ResMut,
    };

    use super::*;

    #[derive(Debug)]
    struct Player;
    #[derive(Debug)]
    struct Health(usize);

    #[test]
    fn insert_entity() {
        let mut ecs = Ecs::new();
        assert_eq!(ecs.entity_count(), 0);
        ecs.insert((Player, Health(10)));
        assert_eq!(ecs.entity_count(), 1);
        ecs.insert((Player, Health(10)));
        ecs.insert((Player, Health(10)));
        assert_eq!(ecs.entity_count(), 3);
    }

    #[test]
    fn insert_entities_with_relationship() {
        struct ChildOf;
        #[derive(Debug)]
        struct Hat;

        let mut ecs = Ecs::new();
        assert_eq!(ecs.entity_count(), 0);
        let player = ecs.insert((Player, Health(10)));
        ecs.insert((Hat,));
        let hat = ecs.insert((Hat,));
        let hat2 = ecs.insert((Hat,));
        ecs.insert_relationship::<ChildOf>(hat, player);
        ecs.insert_relationship::<ChildOf>(hat2, player);

        let q = Q::<(&Hat,)>::new(&ecs.entity_store, &ecs.relationship_store)
            .with_relationship::<ChildOf>(player);
        assert_eq!(q.iter().count(), 2);
    }

    #[test]
    fn run_system_adding_entity() {
        let mut ecs = Ecs::new();

        assert_eq!(ecs.entity_count(), 0);
        let add_entity = |command_buffer: &CommandBuffer| {
            command_buffer.insert((Player, Health(10)));
            command_buffer.insert((Player, Health(9)));
        };

        let mut system_set = SystemSet::new();
        system_set.add_system(add_entity);
        ecs.register_system_set(system_set);
        ecs.run_systems();
        assert_eq!(ecs.pending_commands.len(), 2);

        ecs.execute_pending_commands();
        assert_eq!(ecs.pending_commands.len(), 0);
        assert_eq!(ecs.entity_count(), 2);
    }

    #[test]
    fn run_system_emitting_event() {
        struct ExitEvent;
        let mut ecs = Ecs::new();
        assert!(ecs.event_queue.is_empty());

        let emit_exit_event = |event_writer: EventWriter<ExitEvent>| {
            event_writer.write(ExitEvent);
        };

        let mut system_set = SystemSet::new();
        system_set.add_system(emit_exit_event);
        ecs.register_system_set(system_set);
        ecs.run_systems();

        assert!(!ecs.event_queue.is_empty());
    }

    #[test]
    fn run_system_reading_event() {
        struct EventCount(pub usize);
        struct AEvent;

        let mut ecs = Ecs::new();
        ecs.insert_resource(EventCount(0));
        let write_events = move |event_writer: EventWriter<AEvent>| {
            event_writer.write(AEvent);
            event_writer.write(AEvent);
        };

        let read_events = move |event_reader: EventReader<AEvent>,
                                event_count: ResMut<EventCount>| {
            let ResMut(mut event_count) = event_count;
            event_count.0 += event_reader.iter().count();
        };

        let mut system_set = SystemSet::new();
        system_set.add_system(write_events);
        system_set.add_system(read_events);
        ecs.register_system_set(system_set);
        ecs.run_systems();
        {
            let event_count = ecs.resource::<EventCount>().unwrap();
            let event_count = event_count.0;
            assert_eq!(event_count, 0);
        }
        ecs.run_systems();
        let event_count = ecs.resource::<EventCount>().unwrap();
        let event_count = event_count.0;
        assert_eq!(event_count, 2);
    }

    #[test]
    fn store_resource() {
        struct Turn(pub u32);
        let mut ecs = Ecs::new();
        ecs.insert_resource(Turn(0));

        let increment_turn_system = |res_turn: ResMut<Turn>| {
            let ResMut(mut turn) = res_turn;
            turn.0 += 1;
        };
        let mut system_set = SystemSet::new();
        system_set.add_system(increment_turn_system);
        ecs.register_system_set(system_set);
        ecs.run_systems();

        let turn = ecs.resource::<Turn>().unwrap();
        assert_eq!(turn.0, 1);
    }
}
