use std::{
    any::Any,
    cell::RefCell,
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::{
    entity::{EntityBundle, EntityId},
    relationship::{Relationship, RelationshipId},
    system::{Into, System, SystemSet},
    Ecs, EntityDefinition,
};

pub struct CommandBuffer {
    next_entity_id: AtomicUsize,
    commands: Rc<RefCell<Vec<Box<dyn Command>>>>,
}

impl CommandBuffer {
    #[must_use]
    pub fn new(next_entity_id: usize) -> Self {
        Self {
            next_entity_id: AtomicUsize::new(next_entity_id),
            commands: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn clear(&mut self) {
        self.commands.borrow_mut().clear();
    }

    pub fn insert_bundle(&self, entity_bundle: EntityBundle) -> EntityId {
        let mut entity_ids = vec![];
        for entity in entity_bundle.entities {
            entity_ids.push(self.insert(entity));
        }

        for (source, relationship_map) in entity_bundle.relationships.iter().enumerate() {
            let relationship_ids = relationship_map.keys().collect::<Vec<_>>();
            for &relationship_id in relationship_ids {
                for &target in &relationship_map[&relationship_id] {
                    self.insert_relationship_with_relationship_id(
                        relationship_id,
                        entity_ids[source],
                        entity_ids[target],
                    );
                }
            }
        }

        entity_ids[entity_bundle.root]
    }

    pub fn insert<ED>(&self, entity: ED) -> EntityId
    where
        ED: 'static + EntityDefinition,
    {
        self.commands
            .borrow_mut()
            .push(Box::new(InsertEntity::new(entity)));
        self.next_entity_id.fetch_add(1, Ordering::Relaxed)
    }

    pub fn add_component<C: 'static>(&self, entity_id: EntityId, component: C) {
        self.commands
            .borrow_mut()
            .push(Box::new(InsertComponent::new(entity_id, component)));
    }

    pub fn insert_relationship<R>(&self, source: EntityId, target: EntityId)
    where
        R: Relationship,
    {
        self.insert_relationship_with_relationship_id(R::relationship_id(), source, target);
    }

    fn insert_relationship_with_relationship_id(
        &self,
        relationship_id: RelationshipId,
        source: EntityId,
        target: EntityId,
    ) {
        self.commands
            .borrow_mut()
            .push(Box::new(InsertRelationship::new(
                relationship_id,
                source,
                target,
            )));
    }

    pub fn insert_resource<R>(&self, resource: R)
    where
        R: 'static + Any,
    {
        self.commands
            .borrow_mut()
            .push(Box::new(InsertResource::new(resource)));
    }

    pub fn register_system_set(&self, system_set: SystemSet) {
        self.commands
            .borrow_mut()
            .push(Box::new(RegisterSystemSet::new(system_set)));
    }

    pub fn register_system<S, M, ST>(&self, system: S)
    where
        S: Into<M, SystemType = ST>,
        ST: System + 'static,
    {
        self.commands
            .borrow_mut()
            .push(Box::new(RegisterSystem::new(system)));
    }

    pub fn flush_commands(&mut self) -> Vec<Box<dyn Command>> {
        self.commands.borrow_mut().drain(..).collect::<Vec<_>>()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.commands.borrow().len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait Command {
    fn apply(&mut self, ecs: &mut Ecs);
}

pub struct InsertEntity<ED>
where
    ED: EntityDefinition,
{
    entity: Option<ED>,
}

impl<ED> InsertEntity<ED>
where
    ED: EntityDefinition,
{
    pub fn new(entity: ED) -> Self {
        Self {
            entity: Some(entity),
        }
    }
}

impl<ED> Command for InsertEntity<ED>
where
    ED: EntityDefinition,
{
    fn apply(&mut self, ecs: &mut Ecs) {
        ecs.insert(self.entity.take().unwrap());
    }
}

pub struct InsertComponent<C> {
    entity_id: EntityId,
    component: Option<C>,
}

impl<C> InsertComponent<C> {
    pub fn new(entity_id: EntityId, component: C) -> Self {
        InsertComponent {
            entity_id,
            component: Some(component),
        }
    }
}

impl<C> Command for InsertComponent<C>
where
    C: 'static,
{
    fn apply(&mut self, ecs: &mut Ecs) {
        ecs.insert_component(self.entity_id, self.component.take().unwrap());
    }
}

pub struct InsertRelationship {
    relationship_id: RelationshipId,
    source: EntityId,
    target: EntityId,
}

impl InsertRelationship {
    #[must_use]
    pub fn new(relationship_id: RelationshipId, source: EntityId, target: EntityId) -> Self
where {
        Self {
            relationship_id,
            source,
            target,
        }
    }
}

impl Command for InsertRelationship {
    fn apply(&mut self, ecs: &mut Ecs) {
        ecs.insert_relationship(self.relationship_id, self.source, self.target);
    }
}

pub struct InsertResource<R> {
    resource: Option<R>,
}

impl<R> InsertResource<R> {
    pub fn new(resource: R) -> Self {
        Self {
            resource: Some(resource),
        }
    }
}

impl<R> Command for InsertResource<R>
where
    R: 'static + Any,
{
    fn apply(&mut self, ecs: &mut Ecs) {
        ecs.insert_resource(self.resource.take().expect("Missing resource"));
    }
}

pub struct RegisterSystemSet {
    system_set: Option<SystemSet>,
}

impl RegisterSystemSet {
    #[must_use]
    pub fn new(system_set: SystemSet) -> Self {
        Self {
            system_set: Some(system_set),
        }
    }
}

impl Command for RegisterSystemSet {
    fn apply(&mut self, ecs: &mut Ecs) {
        ecs.register_system_set(self.system_set.take().unwrap());
    }
}

pub struct RegisterSystem {
    system: Option<Box<dyn System>>,
}

impl RegisterSystem {
    pub fn new<S, ST, M>(system: S) -> Self
    where
        S: Into<M, SystemType = ST>,
        ST: System + 'static,
    {
        Self {
            system: Some(Box::new(Into::into(system))),
        }
    }
}

impl Command for RegisterSystem {
    fn apply(&mut self, ecs: &mut Ecs) {
        let mut system_set = SystemSet::new();
        system_set.add_system(self.system.take().unwrap());
        ecs.register_system_set(system_set);
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
    fn apply_insert_entity_command() {
        let mut ecs = Ecs::new();
        assert_eq!(ecs.entity_count(), 0);
        let mut insert_entity_command = InsertEntity::new((Player, Health(10)));
        insert_entity_command.apply(&mut ecs);
        assert_eq!(ecs.entity_count(), 1);
        let mut insert_entity_command = InsertEntity::new((Player, Health(10)));
        insert_entity_command.apply(&mut ecs);
        let mut insert_entity_command = InsertEntity::new((Player, Health(10)));
        insert_entity_command.apply(&mut ecs);
        assert_eq!(ecs.entity_count(), 3);
    }
}
