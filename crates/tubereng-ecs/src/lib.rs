#![warn(clippy::pedantic)]

use log::trace;
use std::fmt::Debug;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use commands::CommandBuffer;
use system::System;

pub mod commands;
pub mod system;

pub type EntityId = usize;

type ComponentStore = Vec<Option<Box<dyn Any>>>;

pub struct Ecs {
    components: HashMap<TypeId, ComponentStore>,
    next_entity_id: EntityId,
    pending_commands: CommandBuffer,
}
impl Ecs {
    #[must_use]
    pub fn new() -> Ecs {
        Self {
            components: HashMap::new(),
            next_entity_id: 0,
            pending_commands: CommandBuffer::new(),
        }
    }

    #[must_use]
    pub fn entity_count(&self) -> usize {
        self.next_entity_id
    }

    fn allocate_entity(&mut self) -> EntityId {
        let id = self.next_entity_id;
        self.next_entity_id += 1;
        trace!("Allocated entity {}", id);
        id
    }

    pub fn insert<ED>(&mut self, entity: ED) -> EntityId
    where
        ED: EntityDefinition,
    {
        trace!("Inserting entity {:?}", &entity);
        let entity_id = self.allocate_entity();
        entity.write_into_ecs(self, entity_id);
        entity_id
    }

    fn write_component<C: 'static>(&mut self, entity_id: EntityId, component: C) {
        assert!(
            entity_id < self.next_entity_id,
            "Tried to write a component in a unallocated entity"
        );

        let component_store = self
            .components
            .entry(TypeId::of::<C>())
            .or_insert_with(Vec::new);

        component_store.resize_with(self.next_entity_id, || None);
        component_store[entity_id] = Some(Box::new(component));
    }

    pub fn run_systems(&mut self, systems: &[&System]) {
        for system in systems {
            system.run(&mut self.pending_commands);
        }
    }

    pub fn execute_pending_commands(&mut self) {
        let mut pending_commands = CommandBuffer::new();
        std::mem::swap(&mut self.pending_commands, &mut pending_commands);
        for command in pending_commands.iter_mut() {
            command.apply(self);
        }
    }
}

impl Default for Ecs {
    fn default() -> Self {
        Self::new()
    }
}

pub trait EntityDefinition: Debug {
    fn write_into_ecs(self, ecs: &mut Ecs, entity_id: EntityId);
}
impl<A: 'static + Debug, B: 'static + Debug> EntityDefinition for (A, B) {
    fn write_into_ecs(self, ecs: &mut Ecs, entity_id: EntityId) {
        ecs.write_component(entity_id, self.0);
        ecs.write_component(entity_id, self.1);
    }
}

#[cfg(test)]
mod tests {
    use crate::system::Into;

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
    fn run_system_adding_entity() {
        let mut ecs = Ecs::new();

        assert_eq!(ecs.entity_count(), 0);
        let add_entity = |command_buffer: &mut CommandBuffer| {
            command_buffer.insert((Player, Health(10)));
            command_buffer.insert((Player, Health(9)));
        };

        ecs.run_systems(&[&add_entity.into_system()]);
        assert_eq!(ecs.pending_commands.len(), 2);

        ecs.execute_pending_commands();
        assert_eq!(ecs.pending_commands.len(), 0);
        assert_eq!(ecs.entity_count(), 2);
    }
}
