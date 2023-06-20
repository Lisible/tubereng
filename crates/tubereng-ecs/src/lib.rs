#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use entity::EntityStore;
use log::{info, trace};
use std::fmt::Debug;
use system::{ExecutionContext, System, SystemSet};

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
    system_sets: Vec<SystemSet>,
}
impl Ecs {
    #[must_use]
    pub fn new() -> Ecs {
        Self {
            entity_store: EntityStore::new(),
            pending_commands: CommandBuffer::new(),
            setup_system: None,
            system_sets: vec![],
        }
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
            };
            setup_system.execute(&ctx);
        }
    }

    pub fn run_systems(&mut self) {
        for system_set in &mut self.system_sets {
            for system in system_set.iter_mut() {
                let ctx = ExecutionContext {
                    command_buffer: &self.pending_commands,
                    entity_store: &self.entity_store,
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

        let mut system_set = SystemSet::new();
        system_set.add_system(add_entity);
        ecs.register_system_set(system_set);
        ecs.run_systems();
        assert_eq!(ecs.pending_commands.len(), 2);

        ecs.execute_pending_commands();
        assert_eq!(ecs.pending_commands.len(), 0);
        assert_eq!(ecs.entity_count(), 2);
    }
}
