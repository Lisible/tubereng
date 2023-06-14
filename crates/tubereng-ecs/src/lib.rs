#![warn(clippy::pedantic)]

use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

pub mod commands;
pub mod system;

pub type EntityId = usize;

type ComponentStore = Vec<Option<Box<dyn Any>>>;

pub struct Ecs {
    components: HashMap<TypeId, ComponentStore>,
    next_entity_id: EntityId,
}
impl Ecs {
    #[must_use]
    pub fn new() -> Ecs {
        Self {
            components: HashMap::new(),
            next_entity_id: 0,
        }
    }

    #[must_use]
    pub fn entity_count(&self) -> usize {
        self.next_entity_id
    }

    fn allocate_entity(&mut self) -> EntityId {
        let id = self.next_entity_id;
        self.next_entity_id += 1;
        id
    }

    pub fn insert<ED>(&mut self, entity: ED) -> EntityId
    where
        ED: EntityDefinition,
    {
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
}

impl Default for Ecs {
    fn default() -> Self {
        Self::new()
    }
}

pub trait EntityDefinition {
    fn write_into_ecs(self, ecs: &mut Ecs, entity_id: EntityId);
}
impl<A: 'static, B: 'static> EntityDefinition for (A, B) {
    fn write_into_ecs(self, ecs: &mut Ecs, entity_id: EntityId) {
        ecs.write_component(entity_id, self.0);
        ecs.write_component(entity_id, self.1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Player;
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
}
