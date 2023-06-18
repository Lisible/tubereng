#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use entity::EntityStore;
use std::fmt::Debug;
use system::{ExecutionContext, Into, System};

use commands::CommandBuffer;

pub mod commands;
pub mod entity;
pub mod query;
pub mod system;

pub type EntityId = usize;

pub struct Ecs {
    entity_store: EntityStore,
    pending_commands: CommandBuffer,
    setup_system: Option<Box<dyn System>>,
    systems: Vec<Box<dyn System>>,
}
impl Ecs {
    #[must_use]
    pub fn new() -> Ecs {
        Self {
            entity_store: EntityStore::new(),
            pending_commands: CommandBuffer::new(),
            setup_system: None,
            systems: vec![],
        }
    }

    pub fn register_setup_system(&mut self, setup_system: Box<dyn System>) {
        self.setup_system = Some(setup_system);
    }

    pub fn register_system<S, M, ST>(&mut self, system: S)
    where
        S: Into<M, SystemType = ST>,
        ST: 'static + System,
    {
        self.systems.push(Box::new(Into::into(system)));
    }

    pub fn run_setup_system(&mut self) {
        if let Some(mut setup_system) = self.setup_system.take() {
            let ctx = ExecutionContext {
                command_buffer: &self.pending_commands,
                entity_store: &self.entity_store,
            };
            setup_system.run(&ctx);
        }
    }

    pub fn run_systems(&mut self) {
        for system in &mut self.systems {
            let ctx = ExecutionContext {
                command_buffer: &self.pending_commands,
                entity_store: &self.entity_store,
            };
            system.run(&ctx);
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

    pub fn execute_pending_commands(&mut self) {
        let mut pending_commands = CommandBuffer::new();
        std::mem::swap(&mut self.pending_commands, &mut pending_commands);
        for mut command in pending_commands.flush_commands() {
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
    fn write_into_entity_store(self, entity_store: &mut EntityStore, entity_id: EntityId);
}
impl<A: 'static + Debug, B: 'static + Debug> EntityDefinition for (A, B) {
    fn write_into_entity_store(self, entity_store: &mut EntityStore, entity_id: EntityId) {
        entity_store.write_component(entity_id, self.0);
        entity_store.write_component(entity_id, self.1);
    }
}

#[cfg(test)]
mod tests {

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
        let add_entity = |command_buffer: &CommandBuffer| {
            command_buffer.insert((Player, Health(10)));
            command_buffer.insert((Player, Health(9)));
        };

        ecs.register_system(add_entity);
        ecs.run_systems();
        assert_eq!(ecs.pending_commands.len(), 2);

        ecs.execute_pending_commands();
        assert_eq!(ecs.pending_commands.len(), 0);
        assert_eq!(ecs.entity_count(), 2);
    }
}
