use std::{any::Any, cell::RefCell, marker::PhantomData, rc::Rc};

use crate::{
    entity::EntityId,
    relationship::Relationship,
    system::{Into, System, SystemSet},
    Ecs, EntityDefinition,
};

pub struct CommandBuffer {
    commands: Rc<RefCell<Vec<Box<dyn Command>>>>,
}

impl CommandBuffer {
    #[must_use]
    pub fn new() -> Self {
        Self {
            commands: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn clear(&mut self) {
        self.commands.borrow_mut().clear();
    }

    pub fn insert<ED>(&self, entity: ED)
    where
        ED: 'static + EntityDefinition,
    {
        self.commands
            .borrow_mut()
            .push(Box::new(InsertEntity::new(entity, |_, _| {})));
    }

    pub fn insert_and_then<ED, F>(&self, entity: ED, callback: F)
    where
        ED: 'static + EntityDefinition,
        F: 'static + Fn(EntityId, &CommandBuffer),
    {
        self.commands
            .borrow_mut()
            .push(Box::new(InsertEntity::new(entity, callback)));
    }

    pub fn insert_relationship<R>(&self, source: EntityId, target: EntityId)
    where
        R: Relationship,
    {
        self.commands
            .borrow_mut()
            .push(Box::new(InsertRelationship::<R>::new(source, target)));
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

impl Default for CommandBuffer {
    fn default() -> Self {
        Self::new()
    }
}

pub trait Command {
    fn apply(&mut self, ecs: &mut Ecs);
}

pub struct InsertEntity<ED, F>
where
    ED: EntityDefinition,
    F: Fn(EntityId, &CommandBuffer),
{
    entity: Option<ED>,
    callback: F,
}

impl<ED, F> InsertEntity<ED, F>
where
    ED: EntityDefinition,
    F: Fn(EntityId, &CommandBuffer),
{
    pub fn new(entity: ED, callback: F) -> Self {
        Self {
            entity: Some(entity),
            callback,
        }
    }
}

impl<ED, F> Command for InsertEntity<ED, F>
where
    ED: EntityDefinition,
    F: Fn(EntityId, &CommandBuffer),
{
    fn apply(&mut self, ecs: &mut Ecs) {
        let id = ecs.insert(self.entity.take().unwrap());
        (self.callback)(id, &ecs.pending_commands);
    }
}

pub struct InsertRelationship<R> {
    source: EntityId,
    target: EntityId,
    _marker: PhantomData<R>,
}

impl<R> InsertRelationship<R> {
    #[must_use]
    pub fn new(source: EntityId, target: EntityId) -> Self
    where
        R: Relationship,
    {
        Self {
            source,
            target,
            _marker: PhantomData,
        }
    }
}

impl<R> Command for InsertRelationship<R>
where
    R: Relationship,
{
    fn apply(&mut self, ecs: &mut Ecs) {
        ecs.insert_relationship::<R>(self.source, self.target);
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
        let mut insert_entity_command = InsertEntity::new((Player, Health(10)), |_, _| {});
        insert_entity_command.apply(&mut ecs);
        assert_eq!(ecs.entity_count(), 1);
        let mut insert_entity_command = InsertEntity::new((Player, Health(10)), |_, _| {});
        insert_entity_command.apply(&mut ecs);
        let mut insert_entity_command = InsertEntity::new((Player, Health(10)), |_, _| {});
        insert_entity_command.apply(&mut ecs);
        assert_eq!(ecs.entity_count(), 3);
    }
}
