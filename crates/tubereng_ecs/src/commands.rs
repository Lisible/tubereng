use std::{
    cell::RefCell,
    marker::PhantomData,
    sync::atomic::{self, AtomicUsize},
    vec::IntoIter,
};

use crate::{Ecs, EntityDefinition, EntityId};

pub struct CommandQueue {
    allocated_entity_count: AtomicUsize,
    next_entity_id: usize,
    deleted_entities: Vec<EntityId>,
    commands: RefCell<Vec<Box<dyn Command>>>,
}
impl CommandQueue {
    #[must_use]
    pub fn new(next_entity_id: usize, deleted_entities: &[EntityId]) -> Self {
        Self {
            allocated_entity_count: AtomicUsize::new(0),
            next_entity_id,
            deleted_entities: deleted_entities.to_vec(),
            commands: RefCell::new(vec![]),
        }
    }
    fn compute_next_entity_id(&self) -> EntityId {
        let allocated_entity_count = self.allocated_entity_count.load(atomic::Ordering::Relaxed);
        let id = if allocated_entity_count < self.deleted_entities.len() {
            self.deleted_entities[allocated_entity_count]
        } else {
            self.next_entity_id + allocated_entity_count - self.deleted_entities.len()
        };
        self.allocated_entity_count
            .fetch_add(1, atomic::Ordering::Relaxed);
        id
    }

    pub fn insert<ED>(&self, entity_definition: ED) -> EntityId
    where
        ED: 'static + EntityDefinition,
    {
        self.push_command(InsertEntity::new(entity_definition));
        self.compute_next_entity_id()
    }

    pub fn insert_component<C: 'static>(&self, entity_id: EntityId, component: C) {
        self.push_command(InsertComponent::new(entity_id, component));
    }

    pub fn remove_component<C: 'static>(&self, entity_id: EntityId) {
        self.push_command(RemoveComponent::<C>::new(entity_id));
    }

    pub fn delete(&self, entity_id: EntityId) {
        self.push_command(DeleteEntity::new(entity_id));
    }

    pub fn insert_resource<R>(&self, resource: R)
    where
        R: 'static,
    {
        self.push_command(InsertResource::new(resource));
    }

    pub fn insert_relationship<R: 'static>(&self, source: EntityId, target: EntityId) {
        self.push_command(InsertRelationship::<R>::new(source, target));
    }

    fn push_command<C>(&self, command: C)
    where
        C: 'static + Command,
    {
        self.commands.borrow_mut().push(Box::new(command));
    }
}

impl IntoIterator for CommandQueue {
    type Item = Box<dyn Command>;
    type IntoIter = IntoIter<Box<dyn Command>>;

    fn into_iter(self) -> Self::IntoIter {
        self.commands.into_inner().into_iter()
    }
}

pub trait Command {
    fn apply(&mut self, ecs: &mut Ecs);
}

pub struct InsertEntity {
    entity_definition: Option<Box<dyn EntityDefinition>>,
}
impl InsertEntity {
    pub fn new<ED>(entity_definition: ED) -> Self
    where
        ED: 'static + EntityDefinition,
    {
        Self {
            entity_definition: Some(Box::new(entity_definition)),
        }
    }
}

impl Command for InsertEntity {
    fn apply(&mut self, ecs: &mut Ecs) {
        let boxed_ed = self.entity_definition.take().unwrap();
        ecs.insert(boxed_ed);
    }
}

pub struct InsertComponent<C> {
    entity_id: EntityId,
    component: Option<C>,
}

impl<C> InsertComponent<C> {
    pub fn new(entity_id: EntityId, component: C) -> Self {
        Self {
            entity_id,
            component: Some(component),
        }
    }
}

impl<C: 'static> Command for InsertComponent<C> {
    fn apply(&mut self, ecs: &mut Ecs) {
        ecs.insert_component(self.entity_id, self.component.take().unwrap());
    }
}

pub struct RemoveComponent<C> {
    entity_id: EntityId,
    _marker: PhantomData<C>,
}

impl<C> RemoveComponent<C> {
    #[must_use]
    pub fn new(entity_id: EntityId) -> Self {
        Self {
            entity_id,
            _marker: PhantomData,
        }
    }
}

impl<C: 'static> Command for RemoveComponent<C> {
    fn apply(&mut self, ecs: &mut Ecs) {
        ecs.remove_component::<C>(self.entity_id);
    }
}

pub struct DeleteEntity {
    entity_id: EntityId,
}

impl DeleteEntity {
    #[must_use]
    pub fn new(entity_id: EntityId) -> Self {
        Self { entity_id }
    }
}

impl Command for DeleteEntity {
    fn apply(&mut self, ecs: &mut Ecs) {
        ecs.delete(self.entity_id);
    }
}

pub struct InsertResource<R>
where
    R: 'static,
{
    resource: Option<R>,
}

impl<R> InsertResource<R>
where
    R: 'static,
{
    pub fn new(resource: R) -> Self {
        Self {
            resource: Some(resource),
        }
    }
}

impl<R> Command for InsertResource<R>
where
    R: 'static,
{
    fn apply(&mut self, ecs: &mut Ecs) {
        ecs.insert_resource(self.resource.take().unwrap());
    }
}

pub struct InsertRelationship<R>
where
    R: 'static,
{
    source: EntityId,
    target: EntityId,
    _marker: PhantomData<R>,
}

impl<R> InsertRelationship<R> {
    #[must_use]
    pub fn new(source: EntityId, target: EntityId) -> Self {
        Self {
            source,
            target,
            _marker: PhantomData,
        }
    }
}

impl<R> Command for InsertRelationship<R>
where
    R: 'static,
{
    fn apply(&mut self, ecs: &mut Ecs) {
        ecs.insert_relationship::<R>(self.source, self.target);
    }
}
